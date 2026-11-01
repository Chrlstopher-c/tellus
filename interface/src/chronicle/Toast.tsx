// Annonce d'un événement de la chronique (apparition, extinction, intervention…).
import { AnimatePresence, motion } from 'framer-motion';
import type { ReactElement } from 'react';

export interface Announcement {
  key: number;
  title: string;
  body: string;
  color: string;
}

export function Toast({ item }: { item: Announcement | null }): ReactElement {
  return (
    <AnimatePresence>
      {item && (
        <motion.div key={item.key} className="toast glass" data-testid="toast"
          initial={{ opacity: 0, y: -8, x: '-50%' }} animate={{ opacity: 1, y: 0, x: '-50%' }}
          exit={{ opacity: 0, y: -6, x: '-50%' }} transition={{ duration: 0.5, ease: [0.22, 1, 0.36, 1] }}>
          <span className="toast-dot" style={{ background: item.color }} />
          <div><p className="toast-title">{item.title}</p><p className="toast-body">{item.body}</p></div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
