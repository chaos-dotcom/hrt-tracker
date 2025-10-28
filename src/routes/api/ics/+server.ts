import type { RequestHandler } from './$types';
import fs from 'fs/promises';

const DATA_FILE = 'hrt-data.json';
const DAY_MS = 24 * 60 * 60 * 1000;

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

export const GET: RequestHandler = async ({ url }) => {
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
		if (!Number.isFinite(next)) continue;

		const step = freqDays * DAY_MS;
		let t = next;

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
