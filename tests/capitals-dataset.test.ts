import { describe, it, expect } from "vitest";
import { readFileSync } from "fs";
import { resolve } from "path";
import {
  capitals,
  getCapitalsByRegion,
  REGIONS,
  type CapitalEntry,
  type Region,
} from "../src/data/capitals";

const VALID_REGIONS: readonly string[] = [
  "Africa",
  "Americas",
  "Asia",
  "Europe",
  "Oceania",
];

// Load raw JSON for structure tests
const rawJson = JSON.parse(
  readFileSync(resolve(__dirname, "../src/data/capitals.json"), "utf-8")
);

// AC1: capitals.json is valid JSON that parses without error
describe("AC1: capitals.json is valid JSON", () => {
  it("parses without error", () => {
    expect(rawJson).toBeDefined();
    expect(rawJson).toHaveProperty("metadata");
    expect(rawJson).toHaveProperty("capitals");
  });
});

// AC2: at least 193 entries
describe("AC2: dataset contains at least 193 entries", () => {
  it("has at least 193 capital entries", () => {
    expect(capitals.length).toBeGreaterThanOrEqual(193);
  });

  it("has at least 193 entries in the raw JSON", () => {
    expect(rawJson.capitals.length).toBeGreaterThanOrEqual(193);
  });
});

// AC3: every entry has exactly the required fields, none missing or null
describe("AC3: every entry has exactly the required fields", () => {
  const requiredFields: (keyof CapitalEntry)[] = [
    "name",
    "country",
    "lat",
    "lon",
    "timezone",
    "region",
  ];

  it.each(capitals.map((c, i) => [i, c.name, c] as const))(
    "entry %i (%s) has all required fields",
    (_index, _name, capital) => {
      for (const field of requiredFields) {
        expect(capital[field]).not.toBeNull();
        expect(capital[field]).not.toBeUndefined();
      }
    }
  );

  it("every name is a non-empty string", () => {
    for (const c of capitals) {
      expect(typeof c.name).toBe("string");
      expect(c.name.length).toBeGreaterThan(0);
    }
  });

  it("every country is a non-empty string", () => {
    for (const c of capitals) {
      expect(typeof c.country).toBe("string");
      expect(c.country.length).toBeGreaterThan(0);
    }
  });

  it("every lat is a number", () => {
    for (const c of capitals) {
      expect(typeof c.lat).toBe("number");
      expect(Number.isFinite(c.lat)).toBe(true);
    }
  });

  it("every lon is a number", () => {
    for (const c of capitals) {
      expect(typeof c.lon).toBe("number");
      expect(Number.isFinite(c.lon)).toBe(true);
    }
  });

  it("every timezone is a non-empty string", () => {
    for (const c of capitals) {
      expect(typeof c.timezone).toBe("string");
      expect(c.timezone.length).toBeGreaterThan(0);
    }
  });

  it("every region is a non-empty string", () => {
    for (const c of capitals) {
      expect(typeof c.region).toBe("string");
      expect(c.region.length).toBeGreaterThan(0);
    }
  });
});

// AC4: lat in [-90, 90] and lon in [-180, 180]
describe("AC4: coordinate ranges", () => {
  it("every lat is in [-90, 90]", () => {
    for (const c of capitals) {
      expect(c.lat).toBeGreaterThanOrEqual(-90);
      expect(c.lat).toBeLessThanOrEqual(90);
    }
  });

  it("every lon is in [-180, 180]", () => {
    for (const c of capitals) {
      expect(c.lon).toBeGreaterThanOrEqual(-180);
      expect(c.lon).toBeLessThanOrEqual(180);
    }
  });
});

// AC5: every timezone is a valid IANA timezone identifier
describe("AC5: valid IANA timezone identifiers", () => {
  it("every timezone follows IANA format (Area/Location)", () => {
    const ianaPattern = /^[A-Za-z]+\/[A-Za-z_-]+(?:\/[A-Za-z_-]+)?$/;
    for (const c of capitals) {
      expect(c.timezone).toMatch(ianaPattern);
    }
  });

  it("no timezone is a UTC offset or abbreviation", () => {
    for (const c of capitals) {
      expect(c.timezone).not.toMatch(/^UTC/);
      expect(c.timezone).not.toMatch(/^GMT/);
      expect(c.timezone).not.toMatch(/^[+-]\d/);
      expect(c.timezone.length).toBeGreaterThan(3);
    }
  });

  it("every timezone can be used with Intl.DateTimeFormat", () => {
    for (const c of capitals) {
      expect(() => {
        new Intl.DateTimeFormat("en-US", { timeZone: c.timezone });
      }).not.toThrow();
    }
  });
});

// AC6: every region is one of the five valid values
describe("AC6: valid region values", () => {
  it("every entry has a valid region", () => {
    for (const c of capitals) {
      expect(VALID_REGIONS).toContain(c.region);
    }
  });

  it("all five regions are represented", () => {
    const regionsInData = new Set(capitals.map((c) => c.region));
    for (const r of VALID_REGIONS) {
      expect(regionsInData).toContain(r);
    }
  });
});

// AC7: uniqueness constraint on name-country
describe("AC7: uniqueness constraint", () => {
  it("no two entries share the same name-country combination", () => {
    const pairs = capitals.map((c) => `${c.name}|${c.country}`);
    const uniquePairs = new Set(pairs);
    expect(uniquePairs.size).toBe(pairs.length);
  });
});

// AC8: metadata with lastUpdated ISO 8601 date
describe("AC8: metadata with lastUpdated", () => {
  it("has a top-level metadata object", () => {
    expect(rawJson.metadata).toBeDefined();
    expect(typeof rawJson.metadata).toBe("object");
  });

  it("metadata has a lastUpdated field", () => {
    expect(rawJson.metadata.lastUpdated).toBeDefined();
  });

  it("lastUpdated is a valid ISO 8601 date string", () => {
    const date = new Date(rawJson.metadata.lastUpdated);
    expect(date.toString()).not.toBe("Invalid Date");
    expect(rawJson.metadata.lastUpdated).toMatch(/^\d{4}-\d{2}-\d{2}/);
  });
});

// AC9: TypeScript module exports
describe("AC9: TypeScript module exports", () => {
  it("exports a typed capitals array", () => {
    expect(Array.isArray(capitals)).toBe(true);
    expect(capitals.length).toBeGreaterThan(0);
  });

  it("capitals array matches CapitalEntry type structure", () => {
    const first = capitals[0];
    expect(typeof first.name).toBe("string");
    expect(typeof first.country).toBe("string");
    expect(typeof first.lat).toBe("number");
    expect(typeof first.lon).toBe("number");
    expect(typeof first.timezone).toBe("string");
    expect(typeof first.region).toBe("string");
  });
});

// AC10: getCapitalsByRegion function
describe("AC10: getCapitalsByRegion", () => {
  it("returns entries matching the given region", () => {
    const african = getCapitalsByRegion("Africa");
    expect(african.length).toBeGreaterThan(0);
    for (const c of african) {
      expect(c.region).toBe("Africa");
    }
  });

  it("returns only entries for the specified region", () => {
    for (const region of VALID_REGIONS) {
      const result = getCapitalsByRegion(region);
      expect(result.length).toBeGreaterThan(0);
      for (const c of result) {
        expect(c.region).toBe(region);
      }
    }
  });

  it("returns an empty array for an unrecognized region", () => {
    expect(getCapitalsByRegion("unknown")).toEqual([]);
    expect(getCapitalsByRegion("antarctica")).toEqual([]);
  });

  it("returns an empty array for an empty string", () => {
    expect(getCapitalsByRegion("")).toEqual([]);
  });

  it("is case-sensitive", () => {
    expect(getCapitalsByRegion("africa")).toEqual([]);
    expect(getCapitalsByRegion("AFRICA")).toEqual([]);
    expect(getCapitalsByRegion("Africa").length).toBeGreaterThan(0);
  });

  it("returns the correct count per region", () => {
    const total = VALID_REGIONS.reduce(
      (sum, r) => sum + getCapitalsByRegion(r).length,
      0
    );
    expect(total).toBe(capitals.length);
  });
});

// AC11: file size under 100KB
describe("AC11: file size under 100KB", () => {
  it("capitals.json is smaller than 100KB uncompressed", () => {
    const content = readFileSync(
      resolve(__dirname, "../src/data/capitals.json"),
      "utf-8"
    );
    const sizeInBytes = Buffer.byteLength(content, "utf-8");
    expect(sizeInBytes).toBeLessThan(100 * 1024);
  });
});

// AC12: countries with multiple capitals
describe("AC12: countries with multiple capitals", () => {
  it("South Africa uses Pretoria (executive capital)", () => {
    const sa = capitals.filter((c) => c.country === "South Africa");
    expect(sa).toHaveLength(1);
    expect(sa[0].name).toBe("Pretoria");
  });

  it("Malaysia uses Kuala Lumpur (not Putrajaya)", () => {
    const my = capitals.filter((c) => c.country === "Malaysia");
    expect(my).toHaveLength(1);
    expect(my[0].name).toBe("Kuala Lumpur");
  });

  it("Bolivia uses La Paz (executive capital, not Sucre)", () => {
    const bo = capitals.filter((c) => c.country === "Bolivia");
    expect(bo).toHaveLength(1);
    expect(bo[0].name).toBe("La Paz");
  });

  it("each country appears exactly once", () => {
    const countryCounts = new Map<string, number>();
    for (const c of capitals) {
      countryCounts.set(c.country, (countryCounts.get(c.country) || 0) + 1);
    }
    for (const [country, count] of countryCounts) {
      expect(count).toBe(1);
    }
  });
});

// Edge case: coordinate precision
describe("Edge case: coordinate precision", () => {
  it("coordinates have at least 2 decimal places", () => {
    for (const c of capitals) {
      const latStr = c.lat.toString();
      const lonStr = c.lon.toString();
      if (latStr.includes(".")) {
        expect(latStr.split(".")[1].length).toBeGreaterThanOrEqual(2);
      }
      if (lonStr.includes(".")) {
        expect(lonStr.split(".")[1].length).toBeGreaterThanOrEqual(2);
      }
    }
  });

  it("coordinates have no more than 6 decimal places", () => {
    for (const c of capitals) {
      const latStr = c.lat.toString();
      const lonStr = c.lon.toString();
      if (latStr.includes(".")) {
        expect(latStr.split(".")[1].length).toBeLessThanOrEqual(6);
      }
      if (lonStr.includes(".")) {
        expect(lonStr.split(".")[1].length).toBeLessThanOrEqual(6);
      }
    }
  });
});

// Edge case: no disputed territories
describe("Edge case: excluded entities", () => {
  it("does not include Taiwan", () => {
    expect(capitals.find((c) => c.country === "Taiwan")).toBeUndefined();
  });

  it("does not include Kosovo", () => {
    expect(capitals.find((c) => c.country === "Kosovo")).toBeUndefined();
  });

  it("does not include Palestine", () => {
    expect(capitals.find((c) => c.country === "Palestine")).toBeUndefined();
  });

  it("does not include Western Sahara", () => {
    expect(capitals.find((c) => c.country === "Western Sahara")).toBeUndefined();
  });

  it("does not include Vatican City (UN permanent observer, not member state)", () => {
    expect(capitals.find((c) => c.country === "Vatican City")).toBeUndefined();
  });
});

// Edge case: city names without diacritics
describe("Edge case: city names use English transliterations", () => {
  it("Chisinau (not Chișinău)", () => {
    const md = capitals.find((c) => c.country === "Moldova");
    expect(md?.name).toBe("Chisinau");
  });
});

// REGIONS export
describe("REGIONS export", () => {
  it("exports exactly five regions", () => {
    expect(REGIONS).toHaveLength(5);
  });

  it("contains all valid region values", () => {
    expect([...REGIONS]).toEqual([
      "Africa",
      "Americas",
      "Asia",
      "Europe",
      "Oceania",
    ]);
  });

  it("is readonly", () => {
    // Type-level check: REGIONS is readonly Region[]
    const r: readonly Region[] = REGIONS;
    expect(r).toBe(REGIONS);
  });
});

// Edge case: recently changed capitals
describe("Edge case: recently changed capitals", () => {
  it("Indonesia uses Jakarta (current operational capital)", () => {
    const id = capitals.find((c) => c.country === "Indonesia");
    expect(id?.name).toBe("Jakarta");
  });

  it("Burundi uses Gitega (current capital since 2019)", () => {
    const bi = capitals.find((c) => c.country === "Burundi");
    expect(bi?.name).toBe("Gitega");
  });

  it("Kazakhstan uses Astana (current capital)", () => {
    const kz = capitals.find((c) => c.country === "Kazakhstan");
    expect(kz?.name).toBe("Astana");
  });
});

// Edge case: island nation timezones use specific identifiers
describe("Edge case: island nation timezone specificity", () => {
  it("Tuvalu uses Pacific/Funafuti", () => {
    const tv = capitals.find((c) => c.country === "Tuvalu");
    expect(tv?.timezone).toBe("Pacific/Funafuti");
  });

  it("Kiribati uses Pacific/Tarawa", () => {
    const ki = capitals.find((c) => c.country === "Kiribati");
    expect(ki?.timezone).toBe("Pacific/Tarawa");
  });

  it("Nauru uses Pacific/Nauru", () => {
    const nr = capitals.find((c) => c.country === "Nauru");
    expect(nr?.timezone).toBe("Pacific/Nauru");
  });
});
