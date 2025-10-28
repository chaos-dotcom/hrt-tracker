import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { convertHormone, type Hormone } from '$lib/hormone-convert';

export const POST: RequestHandler = async ({ request }) => {
  const { value, hormone, fromUnit, toUnit } = await request.json() as {
    value: number;
    hormone: Hormone;
    fromUnit: string;
    toUnit: string;
  };

  if (typeof value !== 'number' || !isFinite(value)) {
    return json({ error: 'Invalid value' }, { status: 400 });
  }

  try {
    const converted = convertHormone(value, hormone, fromUnit, toUnit);
    return json({ value: Number(converted.toFixed(3)), unit: toUnit });
  } catch (e: any) {
    return json({ error: e?.message ?? 'Conversion failed' }, { status: 400 });
  }
};
