import capitalsData from "./capitals.json";

export type Region = "Africa" | "Americas" | "Asia" | "Europe" | "Oceania";

export interface CapitalEntry {
  name: string;
  country: string;
  lat: number;
  lon: number;
  timezone: string;
  region: string;
}

export const REGIONS: readonly Region[] = [
  "Africa",
  "Americas",
  "Asia",
  "Europe",
  "Oceania",
] as const;

export const capitals: CapitalEntry[] = capitalsData.capitals;

export function getCapitalsByRegion(region: string): CapitalEntry[] {
  return capitals.filter((capital) => capital.region === region);
}
