import { createQuery } from '@tanstack/solid-query';
import { Accessor } from 'solid-js';
import {
  Aircraft,
  DefaultWorld,
  Flight,
  Points,
  RadioMessage,
  World,
} from './types';
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
export function useAircraft(renderRate: Accessor<number>) {
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

// Flights
export const getFlights = '/api/game/flights';
export function useFlights() {
  return createQuery<Array<Flight>>(() => ({
    queryKey: [getFlights],
    queryFn: async () => {
      const result = await fetch(`${baseAPIPath}${getFlights}`);
      if (!result.ok) return [];
      return result.json();
    },
    initialData: [],
    staleTime: 2000,
    refetchInterval: 2000,
    refetchOnMount: 'always',
    refetchOnReconnect: 'always',
    throwOnError: true, // Throw an error if the query fails
  }));
}

export function useFlight(id: number) {
  return createQuery<Flight | null>(() => ({
    queryKey: [getFlights, id],
    queryFn: async () => {
      const flights = useFlights();
      if (!flights.data) return null;
      return flights.data.find((f) => f.id === id) ?? null;
    },
    staleTime: 2000,
    refetchInterval: 2000,
    refetchOnMount: 'always',
    refetchOnReconnect: 'always',
    throwOnError: true, // Throw an error if the query fails
  }));
}

export function useFlightByAircraft(id: string) {
  return createQuery<Flight | null>(() => ({
    queryKey: [getFlights, id],
    queryFn: async () => {
      const flights = useFlights();
      if (!flights.data) return null;
      return (
        flights.data.find(
          (f) => f.status.type !== 'scheduled' && f.status.value === id
        ) ?? null
      );
    },
    staleTime: 2000,
    refetchInterval: 2000,
    refetchOnMount: 'always',
    refetchOnReconnect: 'always',
    throwOnError: true, // Throw an error if the query fails
  }));
}

export const postCreateFlight = '/api/game/flight';
export const deleteFlight = (id: number) => `/api/game/flight/${id}`;

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
    initialData: DefaultWorld,
    staleTime: Infinity,
    refetchOnReconnect: 'always',
    refetchOnMount: 'always',
    throwOnError: true, // Throw an error if the query fails
  }));
}

export const getPoints = '/api/game/points';
export function usePoints() {
  return createQuery<Points>(() => ({
    queryKey: [getPoints],
    queryFn: async () => {
      const result = await fetch(`${baseAPIPath}${getPoints}`);
      if (!result.ok) return null;
      return result.json();
    },
    staleTime: 1000,
    refetchInterval: 1000,
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
