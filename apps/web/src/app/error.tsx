'use client';

import { useEffect, useTransition } from 'react';
import { useRouter } from 'next/navigation';
import { ErrorState } from '../components/common/States';
import { container } from '../lib/ui';

/** Lỗi khi render một trang (thường do API không phản hồi): giữ thanh đầu/chân trang, "Thử lại" render lại phía máy chủ. */
export default function RouteError({ error, reset }: { error: Error & { digest?: string }; reset: () => void }) {
  const router = useRouter();
  const [pending, startTransition] = useTransition();
  useEffect(() => {
    console.error(error);
  }, [error]);
  return (
    <div className={`${container} max-w-3xl py-16`} aria-busy={pending}>
      <meta name="robots" content="noindex" />
      <ErrorState
        onRetry={() =>
          startTransition(() => {
            router.refresh();
            reset();
          })
        }
      />
    </div>
  );
}
