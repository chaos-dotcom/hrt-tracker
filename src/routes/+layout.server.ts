import fs from 'fs/promises';
import type { LayoutServerLoad } from './$types';
import type { HRTData } from '$lib/types';

const dataFilePath = 'hrt-data.json';

const defaultData: HRTData = {
  bloodTests: [],
  dosageHistory: [],
  measurements: [],
};

export const load: LayoutServerLoad = async () => {
    try {
        const fileContent = await fs.readFile(dataFilePath, 'utf-8');
        const loadedData = JSON.parse(fileContent);
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
