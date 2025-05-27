import { useAtom } from 'solid-jotai';
import { PrimitiveAtom, SetStateAction } from 'jotai';
import { Accessor, createEffect, Resource } from 'solid-js';
import { isSome } from './lib';

type SetAtom<Args extends unknown[], Result> = (...args: Args) => Result;

function getFromLocalStorage(id: string) {
  let json = localStorage.getItem(id);
  if (isSome(json)) {
    return JSON.parse(json);
  } else {
    return null;
  }
}

function setToLocalStorage<T>(id: string, val: T) {
  let json = JSON.stringify(val);
  localStorage.setItem(id, json);
}

export function useStorageAtom<T>(
  id: string,
  atom: PrimitiveAtom<T>
): [
  Resource<Awaited<T>> | Accessor<Awaited<T>>,
  SetAtom<[SetStateAction<T>], void>,
] {
  const name = `storage-${id}`;
  let [theAtom, setTheAtom] = useAtom(atom);

  const local = getFromLocalStorage(name);
  if (isSome(local)) {
    setTheAtom(local);
  }

  createEffect(() => setToLocalStorage(name, theAtom()));

  return [
    theAtom,
    (s) => {
      setTheAtom(s);
      setToLocalStorage(name, theAtom());
    },
  ];
}

import { onCleanup, onMount } from 'solid-js';

export function useGlobalShortcuts(callback: (event: KeyboardEvent) => void) {
  const handler = (event: KeyboardEvent) => {
    const el = document.activeElement;
    const tag = el?.tagName?.toLowerCase();
    const isInput = tag === 'input' || tag === 'textarea';

    if (isInput) return; // Ignore when typing

    callback(event);
  };

  onMount(() => {
    window.addEventListener('keydown', handler);
  });

  onCleanup(() => {
    window.removeEventListener('keydown', handler);
  });
}

export default useGlobalShortcuts;
