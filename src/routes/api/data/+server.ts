import { json } from '@sveltejs/kit';
import fs from 'fs/promises';
import path from 'path';
import type { RequestHandler } from './$types';

const dataFilePath = 'data/hrt-data.json';

export const GET: RequestHandler = async () => {
    try {
        const data = await fs.readFile(dataFilePath, 'utf-8');
        const parsed = JSON.parse(data);
        if (parsed && typeof parsed === 'object' && !Array.isArray(parsed)) {
            // Ensure secrets/settings are not served from JSON
            delete (parsed as any).settings;
        }
        return json(parsed);
    } catch (error: any) {
        if (error.code === 'ENOENT') {
            // File doesn't exist, return default structure
            return json({});
        }
        console.error('Failed to read data file:', error);
        return json({ error: 'Failed to read data' }, { status: 500 });
    }
};

export const POST: RequestHandler = async ({ request }) => {
    try {
        const data = await request.json();
        // Strip settings before persisting JSON
        if (data && typeof data === 'object' && !Array.isArray(data)) {
            delete (data as any).settings;
        }
        // Ensure data directory exists
        await fs.mkdir(path.dirname(dataFilePath), { recursive: true });

        // Atomic write: write to temp file then rename
        const tmpPath = `${dataFilePath}.tmp`;
        await fs.writeFile(tmpPath, JSON.stringify(data, null, 2), 'utf-8');
        await fs.rename(tmpPath, dataFilePath);

        return json({ success: true });
    } catch (error) {
        console.error('Failed to write data file:', error);
        return json({ error: 'Failed to write data' }, { status: 500 });
    }
};
