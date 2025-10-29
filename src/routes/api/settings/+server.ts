import { json } from '@sveltejs/kit';
import fs from 'fs/promises';
import { parse, stringify } from 'yaml';
import type { RequestHandler } from './$types';

const settingsFilePath = 'data/hrt-settings.yaml';

export const GET: RequestHandler = async () => {
  try {
    const yamlText = await fs.readFile(settingsFilePath, 'utf-8');
    const data = parse(yamlText) ?? {};
    if (typeof data !== 'object' || Array.isArray(data)) {
      return json({}, { status: 200 });
    }
    return json(data as Record<string, unknown>);
  } catch (error: any) {
    if (error?.code === 'ENOENT') {
      // No settings file yet; return empty so client uses defaults
      return json({});
    }
    console.error('Failed to read settings file:', error);
    return json({ error: 'Failed to read settings' }, { status: 500 });
  }
};

export const POST: RequestHandler = async ({ request }) => {
  try {
    const data = await request.json();
    const obj = (data && typeof data === 'object' && !Array.isArray(data)) ? data : {};
    const yamlText = stringify(obj);
    await fs.writeFile(settingsFilePath, yamlText, 'utf-8');
    return json({ success: true });
  } catch (error) {
    console.error('Failed to write settings file:', error);
    return json({ error: 'Failed to write settings' }, { status: 500 });
  }
};
