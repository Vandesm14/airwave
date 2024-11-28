import { createQuery } from '@tanstack/solid-query';
import { Accessor } from 'solid-js';
import { Aircraft, Game, RadioMessage, World } from './types';

const defaultURL = `${window.location.protocol}//${window.location.hostname}:9001/api`;
const search = new URLSearchParams(window.location.search);
export const baseAPIPath = search.has('api') ? search.get('api') : defaultURL;

export function useAircraft(renderRate: Accessor<number>) {
  return createQuery<Array<Aircraft>>(() => ({
    queryKey: ['/api/game/aircraft'],
    queryFn: async () => {
      const result = await fetch(`${baseAPIPath}/game/aircraft`);
      if (!result.ok) return [];
      return result.json();
    },
    initialData: [],
    staleTime: renderRate(),
    refetchInterval: renderRate(),
    refetchOnMount: 'always',
    refetchOnReconnect: 'always',
    throwOnError: true, // Throw an error if the query fails
  }));
}

export function useWorld() {
  return createQuery<World>(() => ({
    queryKey: ['/api/world'],
    queryFn: async () => {
      const result = await fetch(`${baseAPIPath}/world`);
      if (!result.ok) return null;
      return result.json();
    },
    staleTime: Infinity,
    refetchOnReconnect: 'always',
    throwOnError: true, // Throw an error if the query fails
  }));
}

export function useGame() {
  return createQuery<Game>(() => ({
    queryKey: ['/api/game'],
    queryFn: async () => {
      const result = await fetch(`${baseAPIPath}/game`);
      if (!result.ok) return null;
      return result.json();
    },
    staleTime: 1000,
    refetchInterval: 1000,
    refetchOnReconnect: 'always',
    throwOnError: true, // Throw an error if the query fails
  }));
}

export function useMessages() {
  return createQuery<Array<RadioMessage>>(() => ({
    queryKey: ['/api/messages'],
    queryFn: async () => {
      const result = await fetch(`${baseAPIPath}/messages`);
      if (!result.ok) return [];
      return result.json();
    },
    initialData: [],
    staleTime: 500,
    refetchInterval: 500,
    refetchOnMount: 'always',
    refetchOnReconnect: 'always',
    throwOnError: true, // Throw an error if the query fails
  }));
}

export function usePing() {
  return createQuery<boolean>(() => ({
    queryKey: ['/api/ping'],
    queryFn: async () => {
      try {
        const result = await fetch(`${baseAPIPath}/ping`);
        if (!result.ok) return false;
        return (await result.text()) === 'pong';
      } catch {
        return false;
      }
    },
    staleTime: 2000,
    refetchInterval: 2000,
    refetchOnMount: 'always',
    refetchOnReconnect: 'always',
    throwOnError: true, // Throw an error if the query fails
  }));
}
