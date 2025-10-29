import { json } from '@sveltejs/kit';
import fs from 'fs/promises';
import type { RequestHandler } from './$types';

const dataFilePath = 'hrt-data.json';

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
        await fs.writeFile(dataFilePath, JSON.stringify(data, null, 2), 'utf-8');
        return json({ success: true });
    } catch (error) {
        console.error('Failed to write data file:', error);
        return json({ error: 'Failed to write data' }, { status: 500 });
    }
};
