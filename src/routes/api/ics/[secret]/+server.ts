import type { RequestHandler } from './$types';
import fs from 'fs/promises';
import { parse } from 'yaml';

const DATA_FILE = 'hrt-data.json';
const DAY_MS = 24 * 60 * 60 * 1000;
const settingsFilePath = 'hrt-settings.yaml';

const pad = (n: number) => String(n).padStart(2, '0');
function toICSDateTime(ms: number): string {
	const d = new Date(ms);
	return `${d.getUTCFullYear()}${pad(d.getUTCMonth() + 1)}${pad(d.getUTCDate())}T${pad(d.getUTCHours())}${pad(d.getUTCMinutes())}${pad(d.getUTCSeconds())}Z`;
}
function escapeText(s: string): string {
	return String(s)
		.replace(/\\/g, '\\\\')
		.replace(/;/g, '\\;')
		.replace(/,/g, '\\,')
		.replace(/\r?\n/g, '\\n');
}

function summaryForMedication(medicationType: string): string {
	switch (medicationType) {
		case 'injectableEstradiol': return 'Injection';
		case 'oralEstradiol': return 'Oral Estradiol';
		case 'antiandrogen': return 'Antiandrogen';
		case 'progesterone': return 'Progesterone';
		default: return 'Medication';
	}
}

function makeEvent(uid: string, startMs: number, summary: string, description?: string): string {
	const lines = [
		'BEGIN:VEVENT',
		`UID:${uid}`,
		`DTSTAMP:${toICSDateTime(Date.now())}`,
		`DTSTART:${toICSDateTime(startMs)}`,
		'DURATION:PT5M',
		`SUMMARY:${escapeText(summary)}`,
		description ? `DESCRIPTION:${escapeText(description)}` : undefined,
		'CATEGORIES:HRT',
		'TRANSP:OPAQUE',
		'END:VEVENT'
	].filter(Boolean) as string[];
	return lines.join('\r\n');
}

function addMonthsUTC(ms: number, months: number): number {
	const d = new Date(ms);
	const y = d.getUTCFullYear();
	const m = d.getUTCMonth();
	const day = d.getUTCDate();
	const hh = d.getUTCHours();
	const mm = d.getUTCMinutes();
	const ss = d.getUTCSeconds();
	// days in target month
	const targetMonthIndex = m + months;
	const lastDayTargetMonth = new Date(Date.UTC(y, targetMonthIndex + 1, 0)).getUTCDate();
	const safeDay = Math.min(day, lastDayTargetMonth);
	const next = new Date(Date.UTC(y, targetMonthIndex, 1, hh, mm, ss));
	next.setUTCDate(safeDay);
	return next.getTime();
}
function setLocalMorning10(ms: number): number {
	const d = new Date(ms);
	d.setHours(10, 0, 0, 0);
	return d.getTime();
}

export const GET: RequestHandler = async ({ params, url }) => {
	// Verify secret from settings
	let configuredSecret = '';
	try {
		const yamlText = await fs.readFile(settingsFilePath, 'utf-8');
		const conf = parse(yamlText) ?? {};
		if (conf && typeof (conf as any).icsSecret === 'string') {
			configuredSecret = ((conf as any).icsSecret as string).trim();
		}
	} catch {
		// missing settings OK
	}
	const provided = (params as { secret?: string }).secret?.trim() ?? '';
	if (!configuredSecret || provided !== configuredSecret) {
		return new Response('Not found', { status: 404 });
	}

	// Options via query params
	const horizonDaysParam = url.searchParams.get('horizonDays');
	const includePastParam = url.searchParams.get('includePast');

	const horizonDays = Number.isFinite(Number(horizonDaysParam)) && Number(horizonDaysParam) > 0
		? Number(horizonDaysParam)
		: 365;

	const includePast = includePastParam === null ? true : includePastParam !== '0';

	let data: any = {};
	try {
		const raw = await fs.readFile(DATA_FILE, 'utf-8');
		data = JSON.parse(raw || '{}');
	} catch (e: any) {
		// if file missing, serve empty calendar
		if (e?.code !== 'ENOENT') {
			console.error('ICS: failed to read data file:', e);
		}
	}

	const now = Date.now();
	const horizonEnd = now + horizonDays * DAY_MS;

	const events: string[] = [];

	// 1) Existing recorded doses (history)
	if (includePast && Array.isArray(data.dosageHistory)) {
		for (const d of data.dosageHistory) {
			if (!d || typeof d.date !== 'number') continue;
			const medicationType = d.medicationType || 'medication';
			const name = d.type || '';
			const qty = d.dose != null ? d.dose : '';
			const unit = d.unit || 'mg';
			const site = d.injectionSite ? `; Site: ${d.injectionSite}` : '';
			const note = d.note ? `; Note: ${d.note}` : '';
			const summary = `${summaryForMedication(medicationType)}: ${name} ${qty} ${unit}`.trim();
			const desc = `Recorded dose${site}${note}`.trim();
			const uid = `${medicationType}-${d.date}-history@hrt-tracker`;
			events.push(makeEvent(uid, d.date, summary, desc));
		}
	}

	// 2) Future scheduled doses (from regimen)
	type RegimenKey = 'injectableEstradiol' | 'oralEstradiol' | 'antiandrogen' | 'progesterone';
	const regimenKeys: RegimenKey[] = ['injectableEstradiol', 'oralEstradiol', 'antiandrogen', 'progesterone'];

	for (const key of regimenKeys) {
		const sched = data?.[key];
		if (!sched) continue;
		const freqDays = Number(sched.frequency);
		const next = Number(sched.nextDoseDate);
		if (!freqDays || !Number.isFinite(freqDays) || freqDays <= 0) continue;

		const step = freqDays * DAY_MS;

		// Determine start time prioritizing last recorded dose; fallback to configured nextDoseDate
		let t: number;

		const lastTakenDates = Array.isArray(data.dosageHistory)
			? data.dosageHistory
					.filter((d: any) => d && d.medicationType === key && typeof d.date === 'number' && isFinite(d.date))
					.map((d: any) => d.date)
			: [];

		if (lastTakenDates.length > 0) {
			const lastTaken = Math.max(...lastTakenDates);
			t = lastTaken + step;
		} else if (Number.isFinite(next)) {
			t = next;
		} else {
			continue;
		}

		// Ensure the first generated event is not before today (UTC) so we keep "today" occurrences
		if (!Number.isFinite(t)) continue;
		{
			const dNow = new Date();
			const todayUTCStart = Date.UTC(dNow.getUTCFullYear(), dNow.getUTCMonth(), dNow.getUTCDate(), 0, 0, 0, 0);
			while (t < todayUTCStart) {
				t += step;
			}
		}

		while (t <= horizonEnd) {
			const name = sched.type || '';
			const qty = sched.dose != null ? sched.dose : '';
			const unit = sched.unit || 'mg';
			const route = key === 'progesterone' && sched.route ? ` (${sched.route})` : '';
			const summary = `Scheduled ${summaryForMedication(key)}${route}: ${name} ${qty} ${unit}`.trim();
			const desc = `Scheduled per regimen; every ${freqDays} day(s).`;
			const uid = `${key}-${t}-scheduled@hrt-tracker`;
			events.push(makeEvent(uid, t, summary, desc));
			t += step;
		}
	}

	// 3) Scheduled blood tests (optional, from settings; start from last recorded test)
	try {
		let enableBlood = false;
		let intervalMonths = 0;
		try {
			const yamlText = await fs.readFile(settingsFilePath, 'utf-8');
			const settings = parse(yamlText) ?? {};
			enableBlood = !!(settings as any).enableBloodTestSchedule;
			intervalMonths = Number((settings as any).bloodTestIntervalMonths);
		} catch {}
		if (enableBlood && Number.isFinite(intervalMonths) && intervalMonths > 0 && Array.isArray(data.bloodTests) && data.bloodTests.length > 0) {
			const lastDates = data.bloodTests
				.filter((b: any) => b && typeof b.date === 'number' && isFinite(b.date))
				.map((b: any) => b.date);
			if (lastDates.length > 0) {
				const last = Math.max(...lastDates);
				let t = addMonthsUTC(last, intervalMonths);
				t = setLocalMorning10(t);
				while (t <= now) {
					t = addMonthsUTC(t, intervalMonths);
					t = setLocalMorning10(t);
				}
				while (t <= horizonEnd) {
					const uid = `bloodtest-${t}-scheduled@hrt-tracker`;
					const summary = 'Scheduled Blood Test';
					const desc = `Routine blood test every ${intervalMonths} month(s).`;
					events.push(makeEvent(uid, t, summary, desc));
					t = addMonthsUTC(t, intervalMonths);
					t = setLocalMorning10(t);
				}
			}
		}
	} catch { /* ignore */ }

	const calendar = [
		'BEGIN:VCALENDAR',
		'PRODID:-//HRT Tracker//EN',
		'VERSION:2.0',
		'CALSCALE:GREGORIAN',
		'METHOD:PUBLISH',
		'X-WR-CALNAME:HRT Doses',
		'X-WR-TIMEZONE:UTC',
		...events,
		'END:VCALENDAR'
	].join('\r\n');

	return new Response(calendar, {
		status: 200,
		headers: {
			'Content-Type': 'text/calendar; charset=utf-8',
			'Cache-Control': 'no-cache'
		}
	});
};
