import {
  createMutation,
  createQuery,
  useQueryClient,
} from '@tanstack/solid-query';
import { Accessor } from 'solid-js';
import fastDeepEqual from 'fast-deep-equal';
import { Aircraft } from '../../bindings/Aircraft';
import { World } from '../../bindings/World';
import { OutgoingCommandReply } from '../../bindings/OutgoingCommandReply';
import { AirportStatus } from '../../bindings/AirportStatus';
import { DefaultAirportStatus } from './lib';

const defaultURL = `${window.location.protocol}//${window.location.hostname}:9001`;
const search = new URLSearchParams(window.location.search);
export const baseAPIPath = search.has('api') ? search.get('api') : defaultURL;

export type ServerTicks = { ticks: number; lastFetch: number };
export type Ping = { connected: boolean; server_ticks: ServerTicks };

// Misc
export const getPing = '/api/ping';
export function usePing() {
  return createQuery<Ping>(() => ({
    queryKey: [getPing],
    queryFn: async () => {
      const server_ticks = { ticks: 0, lastFetch: Date.now() };
      try {
        const result = await fetch(`${baseAPIPath}${getPing}`);
        if (!result.ok) return { connected: false, server_ticks };
        server_ticks.ticks = parseInt(await result.text());
        return { connected: true, server_ticks };
      } catch {
        return { connected: false, server_ticks };
      }
    },
    initialData: {
      connected: false,
      server_ticks: { ticks: 0, lastFetch: Date.now() },
    },
    staleTime: 2000,
    refetchInterval: 2000,
    refetchOnMount: 'always',
    refetchOnReconnect: 'always',
    throwOnError: true,
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
    throwOnError: true,
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
    throwOnError: true,
  }));
}

export const getMessages = '/api/messages';
export function useMessages() {
  return createQuery<Array<OutgoingCommandReply>>(() => ({
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
    throwOnError: true,
  }));
}

export const getAirportStatusKey = `/api/status`;
export const getAirportStatus = (id: string) => `${getAirportStatusKey}/${id}`;
export function useAirportStatus(id: string) {
  return createQuery<AirportStatus>(() => ({
    queryKey: [getAirportStatusKey],
    queryFn: async () => {
      const result = await fetch(`${baseAPIPath}${getAirportStatus(id)}`);
      if (!result.ok) return [];
      return result.json();
    },
    initialData: DefaultAirportStatus(),
    staleTime: 2000,
    refetchInterval: 2000,
    refetchOnMount: 'always',
    refetchOnReconnect: 'always',
    throwOnError: true,
  }));
}

export const postAirportStatusKey = `/api/status`;
export const postAirportStatus = (id: string) =>
  `${postAirportStatusKey}/${id}`;
export function useSetAirportStatus() {
  const client = useQueryClient();

  return createMutation(() => ({
    mutationKey: [postAirportStatusKey],
    mutationFn: async ({ id, status }: { id: string; status: AirportStatus }) =>
      await fetch(`${baseAPIPath}${postAirportStatus(id)}`, {
        method: 'POST',
        body: JSON.stringify(status),
        headers: {
          'Content-Type': 'application/json',
        },
      }),
    onMutate: ({ status }: { id: string; status: AirportStatus }) =>
      client.setQueryData<AirportStatus>([getAirportStatusKey], () => {
        return status;
      }),
    onSettled: () =>
      client.invalidateQueries({ queryKey: [getAirportStatusKey] }),
  }));
}
