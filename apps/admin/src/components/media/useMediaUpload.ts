import { useCallback, useState } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { api, type MediaItem } from '@tkh/shared';
import { errorMessage } from '../../lib/errors';
import { checkImageFile } from '../../lib/media';

export interface UploadItem {
  key: string;
  name: string;
  status: 'uploading' | 'done' | 'error';
  error?: string;
}

/** Hàng đợi tải ảnh lên thư viện: kiểm loại/dung lượng trước, tải tối đa 3 tệp cùng lúc, báo từng tệp. */
export function useMediaUpload() {
  const { t, i18n } = useTranslation();
  const client = useQueryClient();
  const [uploads, setUploads] = useState<UploadItem[]>([]);

  const patch = (key: string, p: Partial<UploadItem>) => setUploads((xs) => xs.map((x) => (x.key === key ? { ...x, ...p } : x)));

  /** Tải các tệp; trả các ảnh tải thành công (đúng thứ tự). */
  const uploadFiles = useCallback(
    async (files: File[]): Promise<MediaItem[]> => {
      const jobs = files.map((file) => ({ file, key: crypto.randomUUID() }));
      setUploads((xs) => [
        ...jobs.map(({ file, key }) => {
          const bad = checkImageFile(file);
          return {
            key,
            name: file.name,
            status: bad ? ('error' as const) : ('uploading' as const),
            error: bad === 'type' ? t('media.errType') : bad === 'size' ? t('media.errSize') : undefined,
          };
        }),
        ...xs,
      ]);
      const results: (MediaItem | undefined)[] = new Array(jobs.length);
      const queue = jobs.map((j, i) => ({ ...j, i })).filter(({ file }) => !checkImageFile(file));
      const worker = async () => {
        for (let job = queue.shift(); job; job = queue.shift()) {
          try {
            results[job.i] = await api.admin.media.upload(job.file);
            patch(job.key, { status: 'done' });
          } catch (err) {
            patch(job.key, { status: 'error', error: errorMessage(err, t, i18n.language) });
          }
        }
      };
      await Promise.all(Array.from({ length: Math.min(3, queue.length) }, worker));
      if (results.some(Boolean)) await client.invalidateQueries({ queryKey: ['admin', 'media'] });
      return results.filter((x): x is MediaItem => !!x);
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [client, t, i18n.language],
  );

  const dismiss = useCallback((key: string) => setUploads((xs) => xs.filter((x) => x.key !== key)), []);
  const clearFinished = useCallback(() => setUploads((xs) => xs.filter((x) => x.status === 'uploading')), []);

  return { uploads, uploadFiles, dismiss, clearFinished };
}
