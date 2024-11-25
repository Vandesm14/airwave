import { useAtom } from 'solid-jotai';
import { PrimitiveAtom, SetStateAction } from 'jotai';
import { Accessor, createEffect, createUniqueId, Resource } from 'solid-js';
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
  atom: PrimitiveAtom<T>
): [
  Resource<Awaited<T>> | Accessor<Awaited<T>>,
  SetAtom<[SetStateAction<T>], void>,
] {
  const name = `storage-${createUniqueId()}`;
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
