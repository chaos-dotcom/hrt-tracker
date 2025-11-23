import type { RequestHandler } from './$types';
import fs from 'fs/promises';
import path from 'path';

const baseDir = 'data/dosage-photos';

function extFromNameOrType(name: string | undefined, type: string | undefined): string {
  if (name && name.includes('.')) return name.split('.').pop()!.toLowerCase();
  if (!type) return 'bin';
  if (type === 'image/jpeg') return 'jpg';
  if (type === 'image/png') return 'png';
  if (type === 'image/webp') return 'webp';
  if (type === 'image/heic') return 'heic';
  return 'bin';
}

export const POST: RequestHandler = async ({ params, request }) => {
  const { entryId } = params;
  if (!entryId) return new Response(JSON.stringify({ error: 'missing entryId' }), { status: 400 });

  const form = await request.formData();
  const files: File[] = [
    ...form.getAll('file').filter((f): f is File => f instanceof File),
    ...form.getAll('photos').filter((f): f is File => f instanceof File)
  ];
  if (files.length === 0) {
    const single = form.get('photo');
    if (single instanceof File) files.push(single);
  }
  if (files.length === 0) {
    return new Response(JSON.stringify({ error: 'no files' }), { status: 400 });
  }

  const dir = path.join(baseDir, entryId);
  await fs.mkdir(dir, { recursive: true });

  const filenames: string[] = [];
  let idx = 0;
  for (const f of files) {
    const ab = await f.arrayBuffer();
    const buf = Buffer.from(ab);
    const ext = extFromNameOrType((f as any).name, f.type);
    const fname = `${Date.now()}_${idx++}.${ext}`;
    const full = path.join(dir, fname);
    await fs.writeFile(full, buf);
    filenames.push(fname);
  }

  return new Response(JSON.stringify({ filenames }), { headers: { 'Content-Type': 'application/json' } });
};
