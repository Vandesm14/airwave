import {
  createMutation,
  createQuery,
  useQueryClient,
} from '@tanstack/solid-query';
import { Accessor } from 'solid-js';
import { Aircraft, RadioMessage, World } from './types';
import fastDeepEqual from 'fast-deep-equal';

const defaultURL = `${window.location.protocol}//${window.location.hostname}:9001`;
const search = new URLSearchParams(window.location.search);
export const baseAPIPath = search.has('api') ? search.get('api') : defaultURL;

// Misc
export const getPing = '/api/ping';
export function usePing() {
  return createQuery<boolean>(() => ({
    queryKey: [getPing],
    queryFn: async () => {
      try {
        const result = await fetch(`${baseAPIPath}${getPing}`);
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

// Aircraft
export const getAircraft = '/api/game/aircraft';
export function useAircraftWithRate(renderRate: Accessor<number>) {
  return createQuery<Array<Aircraft>>(() => ({
    queryKey: [getAircraft],
    queryFn: async () => {
      const result = await fetch(`${baseAPIPath}${getAircraft}`);
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
export function useAircraft() {
  return createQuery<Array<Aircraft>>(() => ({
    queryKey: [getAircraft],
    queryFn: async () => {
      const result = await fetch(`${baseAPIPath}${getAircraft}`);
      if (!result.ok) return [];
      return result.json();
    },
    initialData: [],
  }));
}

// Flights

export const postAcceptFlight = (id: string) =>
  `/api/game/aircraft/${id}/accept`;
export function useAcceptFlight() {
  const client = useQueryClient();

  return createMutation(() => ({
    mutationKey: [`/api/game/aircraft/accept`],
    mutationFn: async (id: string) =>
      await fetch(`${baseAPIPath}${postAcceptFlight(id)}`, {
        method: 'POST',
      }),
    onMutate: (id: string) =>
      client.setQueryData<Array<Aircraft>>([getAircraft], (old) =>
        (old ?? []).map((a) => {
          if (a.id === id) {
            a.accepted = true;
          }

          return a;
        })
      ),
    onSettled: () => client.invalidateQueries({ queryKey: [getAircraft] }),
  }));
}

// State
export const getWorld = '/api/world';
export function useWorld() {
  return createQuery<World>(() => ({
    queryKey: [getWorld],
    queryFn: async () => {
      const result = await fetch(`${baseAPIPath}${getWorld}`);
      if (!result.ok) return null;
      return result.json();
    },
    staleTime: Infinity,
    refetchOnReconnect: 'always',
    throwOnError: true, // Throw an error if the query fails
  }));
}

export const getMessages = '/api/messages';
export function useMessages() {
  return createQuery<Array<RadioMessage>>(() => ({
    queryKey: [getMessages],
    queryFn: async () => {
      const result = await fetch(`${baseAPIPath}${getMessages}`);
      if (!result.ok) return [];
      return result.json();
    },
    reconcile: (oldData, newData) => {
      // Prevent rerenders if the data hasn't changed.
      if (oldData) {
        const isEqual = fastDeepEqual(oldData, newData);
        if (isEqual) {
          return oldData;
        } else {
          return newData;
        }
      } else {
        return newData;
      }
    },
    initialData: [],
    staleTime: 500,
    refetchInterval: 500,
    refetchOnMount: 'always',
    refetchOnReconnect: 'always',
    throwOnError: true, // Throw an error if the query fails
  }));
}
