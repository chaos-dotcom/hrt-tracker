import type { RequestHandler } from './$types';
import fs from 'fs/promises';
import path from 'path';

const baseDir = 'data/dosage-photos';

function contentTypeFromExt(ext: string): string {
  switch (ext.toLowerCase()) {
    case 'jpg':
    case 'jpeg':
      return 'image/jpeg';
    case 'png':
      return 'image/png';
    case 'webp':
      return 'image/webp';
    case 'heic':
      return 'image/heic';
    default:
      return 'application/octet-stream';
  }
}

export const GET: RequestHandler = async ({ params }) => {
  const { entryId, filename } = params;
  if (!entryId || !filename) return new Response('Not found', { status: 404 });
  const full = path.join(baseDir, entryId, filename);
  try {
    const data = await fs.readFile(full);
    const ext = filename.includes('.') ? filename.split('.').pop()! : '';
    return new Response(data, { headers: { 'Content-Type': contentTypeFromExt(ext) } });
  } catch (e: any) {
    if (e.code === 'ENOENT') return new Response('Not found', { status: 404 });
    return new Response('Error', { status: 500 });
  }
};

export const DELETE: RequestHandler = async ({ params }) => {
  const { entryId, filename } = params;
  if (!entryId || !filename) return new Response(JSON.stringify({ error: 'missing params' }), { status: 400 });
  const full = path.join(baseDir, entryId, filename);
  try {
    await fs.unlink(full);
    return new Response(JSON.stringify({ success: true }), { headers: { 'Content-Type': 'application/json' } });
  } catch (e: any) {
    if (e.code === 'ENOENT') return new Response(JSON.stringify({ success: true }), { headers: { 'Content-Type': 'application/json' } });
    return new Response(JSON.stringify({ error: 'delete failed' }), { status: 500 });
  }
};
