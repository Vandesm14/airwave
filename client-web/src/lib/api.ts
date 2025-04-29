import { createQuery } from '@tanstack/solid-query';
import { Accessor } from 'solid-js';
import fastDeepEqual from 'fast-deep-equal';
import { Aircraft } from '../../bindings/Aircraft';
import { World } from '../../bindings/World';
import { OutgoingCommandReply } from '../../bindings/OutgoingCommandReply';
import { ArrivalStatus } from '../../bindings/ArrivalStatus';
import { DepartureStatus } from '../../bindings/DepartureStatus';
import { AirspaceStatus } from '../../bindings/AirspaceStatus';

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

export const getAirspaceStatusKey = `/api/status`;
export const getAirspaceStatus = (id: string) =>
  `${getAirspaceStatusKey}/${id}`;
export function useAirspaceStatus(id: string) {
  return createQuery<AirspaceStatus>(() => ({
    queryKey: [getAirspaceStatusKey],
    queryFn: async () => {
      const result = await fetch(`${baseAPIPath}${getAirspaceStatus(id)}`);
      if (!result.ok) return [];
      return result.json();
    },
    initialData: {
      arrival: 'normal',
      departure: 'normal',
    } as AirspaceStatus,
    staleTime: 2000,
    refetchInterval: 2000,
    refetchOnMount: 'always',
    refetchOnReconnect: 'always',
    throwOnError: true,
  }));
}

export const postArrivalStatusKey = `/api/status/arrival`;
export const postArrivalStatus = (id: string, status: ArrivalStatus) =>
  `${postArrivalStatusKey}/${id}/${status}`;
export function useArrivalStatus() {
  const client = useQueryClient();

  return createMutation(() => ({
    mutationKey: [postArrivalStatusKey],
    mutationFn: async ({ id, status }: { id: string; status: ArrivalStatus }) =>
      await fetch(`${baseAPIPath}${postArrivalStatus(id, status)}`, {
        method: 'POST',
      }),
    onMutate: ({ status }: { id: string; status: ArrivalStatus }) =>
      client.setQueryData<AirspaceStatus>([getAirspaceStatusKey], (old) => {
        if (old) {
          old.arrival = status;
        }
        return old;
      }),
    onSettled: () =>
      client.invalidateQueries({ queryKey: [getAirspaceStatusKey] }),
  }));
}

export const postDepartureStatusKey = `/api/status/departure`;
export const postDepartureStatus = (id: string, status: DepartureStatus) =>
  `${postDepartureStatusKey}/${id}/${status}`;
export function useDepartureStatus() {
  const client = useQueryClient();

  return createMutation(() => ({
    mutationKey: [postDepartureStatusKey],
    mutationFn: async ({
      id,
      status,
    }: {
      id: string;
      status: DepartureStatus;
    }) =>
      await fetch(`${baseAPIPath}${postDepartureStatus(id, status)}`, {
        method: 'POST',
      }),
    onMutate: ({ status }: { id: string; status: DepartureStatus }) =>
      client.setQueryData<AirspaceStatus>([getAirspaceStatusKey], (old) => {
        if (old) {
          old.departure = status;
        }
        return old;
      }),
    onSettled: () =>
      client.invalidateQueries({ queryKey: [getAirspaceStatusKey] }),
  }));
}
