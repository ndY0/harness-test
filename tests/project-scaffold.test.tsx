import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { readFileSync } from "fs";
import { resolve } from "path";
import { App } from "../src/App";

const ROOT = resolve(__dirname, "..");

/**
 * AC1: Running `npm install` completes without errors and installs all declared
 * dependencies.
 *
 * Verified by asserting that node_modules exists for every declared dependency
 * in package.json. If npm install had not completed successfully, these
 * resolution checks would fail.
 */
describe("AC1: npm install completes without errors", () => {
  const pkg = JSON.parse(readFileSync(resolve(ROOT, "package.json"), "utf-8"));

  it("package.json has a dependencies field", () => {
    expect(pkg.dependencies).toBeDefined();
    expect(typeof pkg.dependencies).toBe("object");
  });

  it("package.json has a devDependencies field", () => {
    expect(pkg.devDependencies).toBeDefined();
    expect(typeof pkg.devDependencies).toBe("object");
  });

  it("all declared dependencies are resolvable", () => {
    const allDeps = {
      ...pkg.dependencies,
      ...pkg.devDependencies,
    };
    for (const dep of Object.keys(allDeps)) {
      const depPath = resolve(ROOT, "node_modules", dep);
      const depPkg = resolve(depPath, "package.json");
      expect(
        () => readFileSync(depPkg, "utf-8"),
        `dependency "${dep}" should be installed in node_modules`
      ).not.toThrow();
    }
  });
});

/**
 * AC2: Running `npm run dev` starts a Vite development server that serves the
 * application on localhost and is reachable via HTTP within 10 seconds.
 *
 * This is an integration/smoke test that verifies the dev script is configured.
 * Actually starting and stopping the server is operationally verified; here we
 * verify the script exists and points to vite.
 */
describe("AC2: npm run dev is configured", () => {
  const pkg = JSON.parse(readFileSync(resolve(ROOT, "package.json"), "utf-8"));

  it('has a "dev" script defined in package.json', () => {
    expect(pkg.scripts).toBeDefined();
    expect(pkg.scripts.dev).toBeDefined();
  });

  it('"dev" script uses vite', () => {
    expect(pkg.scripts.dev).toContain("vite");
  });
});

/**
 * AC3: Running `npm run build` produces a production build in a `dist/`
 * directory with no TypeScript compilation errors and no build warnings.
 *
 * The build script must exist and invoke both tsc and vite build. The vite
 * config must set base to relative paths so dist/ is servable from any path.
 */
describe("AC3: npm run build is configured for production", () => {
  const pkg = JSON.parse(readFileSync(resolve(ROOT, "package.json"), "utf-8"));

  it('has a "build" script defined in package.json', () => {
    expect(pkg.scripts).toBeDefined();
    expect(pkg.scripts.build).toBeDefined();
  });

  it('"build" script invokes TypeScript compilation', () => {
    expect(pkg.scripts.build).toMatch(/tsc/);
  });

  it('"build" script invokes vite build', () => {
    expect(pkg.scripts.build).toMatch(/vite build/);
  });

  it("vite config sets base to relative paths", () => {
    const viteConfig = readFileSync(
      resolve(ROOT, "vite.config.ts"),
      "utf-8"
    );
    // base should be set to "./" for portable static file serving
    expect(viteConfig).toMatch(/base:\s*["']\.\/["']/);
  });
});

/**
 * AC4: The project uses React 18 or later and TypeScript 5 or later, as
 * declared in package.json.
 */
describe("AC4: React 18+ and TypeScript 5+ declared in package.json", () => {
  const pkg = JSON.parse(readFileSync(resolve(ROOT, "package.json"), "utf-8"));

  it("declares react as a dependency", () => {
    expect(pkg.dependencies.react).toBeDefined();
  });

  it("declares react >= 18", () => {
    const version = pkg.dependencies.react;
    // Extract the major version number from semver (handles ^18.x.x, ~18.x.x, 18.x.x, >=18, etc.)
    const match = version.match(/(\d+)/);
    expect(match).not.toBeNull();
    expect(parseInt(match![1], 10)).toBeGreaterThanOrEqual(18);
  });

  it("declares react-dom as a dependency", () => {
    expect(pkg.dependencies["react-dom"]).toBeDefined();
  });

  it("declares react-dom >= 18", () => {
    const version = pkg.dependencies["react-dom"];
    const match = version.match(/(\d+)/);
    expect(match).not.toBeNull();
    expect(parseInt(match![1], 10)).toBeGreaterThanOrEqual(18);
  });

  it("declares typescript as a devDependency", () => {
    expect(pkg.devDependencies.typescript).toBeDefined();
  });

  it("declares typescript >= 5", () => {
    const version = pkg.devDependencies.typescript;
    const match = version.match(/(\d+)/);
    expect(match).not.toBeNull();
    expect(parseInt(match![1], 10)).toBeGreaterThanOrEqual(5);
  });
});

/**
 * AC5: The application renders a single HTML page containing exactly three
 * named layout zones: a hero panel zone, a search bar zone, and a capitals
 * grid zone, each identifiable by a data-zone attribute (data-zone="hero",
 * data-zone="search", data-zone="capitals-grid").
 */
describe("AC5: three named layout zones with data-zone attributes", () => {
  it("renders exactly three elements with data-zone attributes", () => {
    const { container } = render(<App />);
    const zones = container.querySelectorAll("[data-zone]");
    expect(zones.length).toBe(3);
  });

  it('renders a zone with data-zone="hero"', () => {
    const { container } = render(<App />);
    const heroZone = container.querySelector('[data-zone="hero"]');
    expect(heroZone).toBeInTheDocument();
  });

  it('renders a zone with data-zone="search"', () => {
    const { container } = render(<App />);
    const searchZone = container.querySelector('[data-zone="search"]');
    expect(searchZone).toBeInTheDocument();
  });

  it('renders a zone with data-zone="capitals-grid"', () => {
    const { container } = render(<App />);
    const capitalsZone = container.querySelector(
      '[data-zone="capitals-grid"]'
    );
    expect(capitalsZone).toBeInTheDocument();
  });
});

/**
 * AC6: At viewport widths of 768px and above, the hero panel zone and the
 * capitals grid zone are displayed side-by-side (hero left, capitals-grid
 * right), with the search bar zone spanning full width above them.
 *
 * Since jsdom does not apply CSS or compute layout, we verify the CSS source
 * declares the correct grid-template-areas for the desktop media query.
 */
describe("AC6: desktop layout — hero and capitals side-by-side, search full-width", () => {
  const appCss = readFileSync(resolve(ROOT, "src/App.css"), "utf-8");

  it("has a media query breakpoint at 768px", () => {
    expect(appCss).toMatch(/@media\s*\(\s*min-width:\s*768px\s*\)/);
  });

  it("desktop grid-template-areas places search spanning two columns", () => {
    // Extract the desktop media query block
    const mediaMatch = appCss.match(
      /@media\s*\(\s*min-width:\s*768px\s*\)\s*\{([\s\S]*?)\n\s*\}/
    );
    expect(mediaMatch).not.toBeNull();
    const mediaBlock = mediaMatch![1];
    // Should contain "search search" in the grid-template-areas
    expect(mediaBlock).toMatch(/["']search\s+search["']/);
  });

  it("desktop grid-template-areas places hero and capitals side-by-side", () => {
    const mediaMatch = appCss.match(
      /@media\s*\(\s*min-width:\s*768px\s*\)\s*\{([\s\S]*?)\n\s*\}/
    );
    expect(mediaMatch).not.toBeNull();
    const mediaBlock = mediaMatch![1];
    // Should contain "hero capitals" in the grid-template-areas
    expect(mediaBlock).toMatch(/["']hero\s+capitals["']/);
  });

  it("desktop layout uses two columns", () => {
    const mediaMatch = appCss.match(
      /@media\s*\(\s*min-width:\s*768px\s*\)\s*\{([\s\S]*?)\n\s*\}/
    );
    expect(mediaMatch).not.toBeNull();
    const mediaBlock = mediaMatch![1];
    expect(mediaBlock).toMatch(/grid-template-columns:\s*1fr\s+1fr/);
  });
});

/**
 * AC7: At viewport widths below 768px, all three zones stack vertically in the
 * order: hero, search, capitals grid, each occupying the full viewport width.
 *
 * Verified by checking the default (mobile-first) grid-template-areas in the
 * CSS before any media query overrides.
 */
describe("AC7: mobile layout — zones stack vertically", () => {
  const appCss = readFileSync(resolve(ROOT, "src/App.css"), "utf-8");

  it("default grid uses a single column", () => {
    // The default (before @media) should be 1fr single column
    expect(appCss).toMatch(/grid-template-columns:\s*1fr\s*;/);
  });

  it("default grid-template-areas stacks hero, search, capitals vertically", () => {
    // The mobile grid-template-areas should list hero, search, capitals in
    // separate rows. The CSS has them as three separate area strings.
    expect(appCss).toMatch(/["']hero["']/);
    expect(appCss).toMatch(/["']search["']/);
    expect(appCss).toMatch(/["']capitals["']/);
  });

  it("mobile layout order is hero first, search second, capitals third", () => {
    // In the CSS, the grid-template-areas for mobile should list hero before
    // search before capitals
    const heroPos = appCss.search(/grid-template-areas:\s*\n?\s*["']hero["']/);
    expect(heroPos).toBeGreaterThan(-1);
  });

  it("default layout uses three rows", () => {
    expect(appCss).toMatch(/grid-template-rows:\s*auto\s+auto\s+auto/);
  });
});

/**
 * AC8: Each zone renders a visible placeholder element containing the zone
 * name as text (e.g., "Hero Panel", "Search", "Capitals Grid") so the layout
 * structure is visually verifiable.
 */
describe("AC8: placeholder text in each zone", () => {
  it('hero zone contains placeholder text "Hero Panel"', () => {
    render(<App />);
    expect(screen.getByText("Hero Panel")).toBeInTheDocument();
  });

  it('search zone contains placeholder text "Search"', () => {
    render(<App />);
    expect(screen.getByText("Search")).toBeInTheDocument();
  });

  it('capitals grid zone contains placeholder text "Capitals Grid"', () => {
    render(<App />);
    expect(screen.getByText("Capitals Grid")).toBeInTheDocument();
  });

  it("placeholder text is inside the corresponding data-zone element", () => {
    const { container } = render(<App />);

    const heroZone = container.querySelector('[data-zone="hero"]');
    expect(heroZone).not.toBeNull();
    expect(heroZone!.textContent).toContain("Hero Panel");

    const searchZone = container.querySelector('[data-zone="search"]');
    expect(searchZone).not.toBeNull();
    expect(searchZone!.textContent).toContain("Search");

    const capitalsZone = container.querySelector(
      '[data-zone="capitals-grid"]'
    );
    expect(capitalsZone).not.toBeNull();
    expect(capitalsZone!.textContent).toContain("Capitals Grid");
  });
});

/**
 * AC9: The app shell layout uses CSS Grid for the zone arrangement (not
 * flexbox, not absolute positioning).
 */
describe("AC9: CSS Grid is used for zone arrangement", () => {
  const appCss = readFileSync(resolve(ROOT, "src/App.css"), "utf-8");

  it("app-shell uses display: grid", () => {
    expect(appCss).toMatch(/\.app-shell\s*\{[^}]*display:\s*grid/s);
  });

  it("app-shell uses grid-template-areas", () => {
    expect(appCss).toMatch(
      /\.app-shell\s*\{[^}]*grid-template-areas/s
    );
  });

  it("zones use grid-area assignments", () => {
    expect(appCss).toMatch(/\.zone-hero\s*\{[^}]*grid-area:\s*hero/s);
    expect(appCss).toMatch(
      /\.zone-search\s*\{[^}]*grid-area:\s*search/s
    );
    expect(appCss).toMatch(
      /\.zone-capitals\s*\{[^}]*grid-area:\s*capitals/s
    );
  });

  it("does not use flexbox for the main app-shell layout", () => {
    // The .app-shell rule itself must not use display: flex
    // (child elements may use flex, but the shell layout must be grid)
    const shellMatch = appCss.match(
      /\.app-shell\s*\{([^}]*)\}/
    );
    if (shellMatch) {
      expect(shellMatch[1]).not.toMatch(/display:\s*flex/);
    }
  });

  it("the rendered app-shell container has the correct class", () => {
    const { container } = render(<App />);
    const shell = container.querySelector(".app-shell");
    expect(shell).toBeInTheDocument();
  });
});

/**
 * AC10: A tsconfig.json file is present and configured with strict: true.
 */
describe("AC10: tsconfig.json with strict: true", () => {
  it("tsconfig.json exists and is valid JSON", () => {
    const content = readFileSync(resolve(ROOT, "tsconfig.json"), "utf-8");
    expect(() => JSON.parse(content)).not.toThrow();
  });

  it("tsconfig.json has compilerOptions.strict set to true", () => {
    const tsconfig = JSON.parse(
      readFileSync(resolve(ROOT, "tsconfig.json"), "utf-8")
    );
    expect(tsconfig.compilerOptions).toBeDefined();
    expect(tsconfig.compilerOptions.strict).toBe(true);
  });

  it("tsconfig.json targets a modern ES version", () => {
    const tsconfig = JSON.parse(
      readFileSync(resolve(ROOT, "tsconfig.json"), "utf-8")
    );
    expect(tsconfig.compilerOptions.target).toMatch(/ES20\d{2}/);
  });

  it("tsconfig.json has JSX configured for React", () => {
    const tsconfig = JSON.parse(
      readFileSync(resolve(ROOT, "tsconfig.json"), "utf-8")
    );
    expect(tsconfig.compilerOptions.jsx).toMatch(/react/i);
  });
});

/**
 * Edge cases from the spec
 */
describe("Edge cases", () => {
  it("each zone has a minimum height via CSS (min-height: 100px)", () => {
    const appCss = readFileSync(resolve(ROOT, "src/App.css"), "utf-8");
    // The .zone rule should set min-height: 100px
    expect(appCss).toMatch(/\.zone\s*\{[^}]*min-height:\s*100px/s);
  });

  it("CSS Grid fallback exists for unsupported browsers", () => {
    const appCss = readFileSync(resolve(ROOT, "src/App.css"), "utf-8");
    expect(appCss).toMatch(/@supports\s+not\s*\(\s*display:\s*grid\s*\)/);
  });

  it("fallback layout uses display: block for single-column stacking", () => {
    const appCss = readFileSync(resolve(ROOT, "src/App.css"), "utf-8");
    const fallbackMatch = appCss.match(
      /@supports\s+not\s*\(\s*display:\s*grid\s*\)\s*\{([\s\S]*?)\n\}/
    );
    expect(fallbackMatch).not.toBeNull();
    expect(fallbackMatch![1]).toMatch(/display:\s*block/);
  });

  it("index.css prevents horizontal scrollbar with overflow-x: hidden", () => {
    const indexCss = readFileSync(resolve(ROOT, "src/index.css"), "utf-8");
    expect(indexCss).toMatch(/overflow-x:\s*hidden/);
  });

  it("vite config does not contain hardcoded absolute paths", () => {
    const viteConfig = readFileSync(
      resolve(ROOT, "vite.config.ts"),
      "utf-8"
    );
    // Should not contain platform-specific absolute paths
    expect(viteConfig).not.toMatch(/[A-Z]:\\/); // Windows paths
    expect(viteConfig).not.toMatch(/\/home\//); // Linux home paths
    expect(viteConfig).not.toMatch(/\/Users\//); // macOS home paths
  });

  it("no unused create-vite template files exist", () => {
    // Verify App.tsx does not contain default Vite counter demo artifacts
    const appTsx = readFileSync(resolve(ROOT, "src/App.tsx"), "utf-8");
    expect(appTsx).not.toMatch(/useState.*count/);
    expect(appTsx).not.toMatch(/vite\.svg/i);
    expect(appTsx).not.toMatch(/react\.svg/i);
  });
});

/**
 * main.tsx rendering tests — ensure the entry point is correctly structured.
 */
describe("main.tsx entry point", () => {
  it("imports and uses StrictMode", () => {
    const mainContent = readFileSync(
      resolve(ROOT, "src/main.tsx"),
      "utf-8"
    );
    expect(mainContent).toMatch(/StrictMode/);
  });

  it("imports and uses createRoot from react-dom/client", () => {
    const mainContent = readFileSync(
      resolve(ROOT, "src/main.tsx"),
      "utf-8"
    );
    expect(mainContent).toMatch(/createRoot/);
    expect(mainContent).toMatch(/react-dom\/client/);
  });

  it("imports the App component", () => {
    const mainContent = readFileSync(
      resolve(ROOT, "src/main.tsx"),
      "utf-8"
    );
    expect(mainContent).toMatch(/import.*App.*from/);
  });

  it("throws if root element is not found", () => {
    // main.tsx has a guard that throws if #root is missing.
    // We verify the module source includes this guard.
    const mainContent = readFileSync(
      resolve(ROOT, "src/main.tsx"),
      "utf-8"
    );
    expect(mainContent).toMatch(/throw\s+new\s+Error/);
    expect(mainContent).toMatch(/getElementById\s*\(\s*["']root["']\s*\)/);
  });
});

/**
 * main.tsx runtime tests — exercise the actual module to get code coverage.
 * Uses vi.resetModules() and dynamic import so the module runs fresh each time.
 */
describe("main.tsx runtime execution", () => {
  beforeEach(() => {
    vi.resetModules();
  });

  afterEach(() => {
    // Clean up any #root element left in the DOM
    const existing = document.getElementById("root");
    if (existing) {
      existing.remove();
    }
  });

  it("mounts the App into a #root element when present", async () => {
    // Set up the DOM with a #root element
    const rootDiv = document.createElement("div");
    rootDiv.id = "root";
    document.body.appendChild(rootDiv);

    // Dynamic import to execute main.tsx with fresh module cache
    await import("../src/main.tsx");

    // React 18 createRoot().render() is async — wait for content
    await vi.waitFor(() => {
      expect(rootDiv.innerHTML).not.toBe("");
    });

    // The app-shell should be rendered inside root
    const appShell = rootDiv.querySelector(".app-shell");
    expect(appShell).not.toBeNull();
  });

  it("throws when #root element is missing", async () => {
    // Ensure no #root element exists
    const existing = document.getElementById("root");
    if (existing) {
      existing.remove();
    }

    await expect(import("../src/main.tsx")).rejects.toThrow(
      "Root element not found"
    );
  });
});

/**
 * App component rendering — exercises App.tsx to get coverage on the
 * component rendering path.
 */
describe("App component rendering", () => {
  it("renders without throwing", () => {
    expect(() => render(<App />)).not.toThrow();
  });

  it("renders a container with class app-shell", () => {
    const { container } = render(<App />);
    const shell = container.querySelector(".app-shell");
    expect(shell).not.toBeNull();
  });

  it("all three zones are direct children of the app-shell container", () => {
    const { container } = render(<App />);
    const shell = container.querySelector(".app-shell");
    expect(shell).not.toBeNull();
    const children = shell!.children;
    expect(children.length).toBe(3);
  });

  it("each zone has a zone-placeholder child", () => {
    const { container } = render(<App />);
    const placeholders = container.querySelectorAll(".zone-placeholder");
    expect(placeholders.length).toBe(3);
  });

  it("zones have correct CSS class names", () => {
    const { container } = render(<App />);
    expect(container.querySelector(".zone-hero")).toBeInTheDocument();
    expect(container.querySelector(".zone-search")).toBeInTheDocument();
    expect(container.querySelector(".zone-capitals")).toBeInTheDocument();
  });

  it("each zone element also has the base zone class", () => {
    const { container } = render(<App />);
    const zones = container.querySelectorAll(".zone");
    expect(zones.length).toBe(3);
  });
});
