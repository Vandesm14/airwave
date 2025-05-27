import { useAtom } from 'solid-jotai';
import { PrimitiveAtom, SetStateAction } from 'jotai';
import { Accessor, createEffect, Resource, Signal } from 'solid-js';
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

export function useStorage<T>(
  id: string,
  signal: Signal<T>,
  options?: {
    serialize?: (data: T) => string;
    deserialize?: (data: string) => T;
  }
): Signal<T> {
  let [theSignal, setTheSignal] = signal;

  const serialize = options?.serialize ?? JSON.stringify;
  const deserialize = options?.deserialize ?? JSON.parse;

  onMount(() => {
    const local = getFromLocalStorage(id);
    if (isSome(local)) {
      setTheSignal(deserialize(local));
    }
  });

  createEffect(() => setToLocalStorage(id, serialize(theSignal())));

  return [theSignal, setTheSignal];
}

export function useStorageAtom<T>(
  id: string,
  atom: PrimitiveAtom<T>,
  options?: {
    serialize?: (data: T) => string;
    deserialize?: (data: string) => T;
  }
): [
  Resource<Awaited<T>> | Accessor<Awaited<T>>,
  SetAtom<[SetStateAction<T>], void>,
] {
  let [theAtom, setTheAtom] = useAtom(atom);

  const serialize = options?.serialize ?? JSON.stringify;
  const deserialize = options?.deserialize ?? JSON.parse;

  onMount(() => {
    const local = getFromLocalStorage(id);
    if (isSome(local)) {
      setTheAtom(deserialize(local));
    }
  });

  const local = getFromLocalStorage(id);
  if (isSome(local)) {
    setTheAtom(deserialize(local));
  }

  createEffect(() => setToLocalStorage(id, serialize(theAtom())));

  return [theAtom, setTheAtom];
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
