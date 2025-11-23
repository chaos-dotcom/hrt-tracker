import fs from 'fs/promises';
import type { LayoutServerLoad } from './$types';
import type { HRTData } from '$lib/types';

const dataFilePath = 'data/hrt-data.json';

const defaultData: HRTData = {
  bloodTests: [],
  dosageHistory: [],
  measurements: [],
  vials: [],
  notes: [],
};

export const load: LayoutServerLoad = async () => {
    try {
        const fileContent = await fs.readFile(dataFilePath, 'utf-8');
        const text = fileContent?.toString() ?? '';
        let loadedData: Partial<HRTData> = {};
        if (text.trim().length > 0) {
            try {
                loadedData = JSON.parse(text);
            } catch {
                console.warn('Data file contains invalid JSON; starting with defaults.');
                loadedData = {};
            }
        }
        return {
            initialHrtData: { ...defaultData, ...loadedData }
        };
    } catch (error: any) {
        if (error.code === 'ENOENT') {
            // File doesn't exist, return default structure
            return {
                initialHrtData: defaultData
            };
        }
        console.error('Failed to read data file:', error);
        // We should probably not crash the app, just return default data.
        return {
            initialHrtData: defaultData
        };
    }
};
