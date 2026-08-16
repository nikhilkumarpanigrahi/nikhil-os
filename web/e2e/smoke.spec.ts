import { expect, test, type Page } from "@playwright/test";

// End-to-end smoke of the Web Alpha critical path:
// landing → real boot sequence → desktop → terminal pipeline → telemetry.
// The WASM core is the only source of truth; every assertion below reflects
// real runtime state, not faked UI values.

// Boot into the desktop from a cold page load. Sidebar buttons are scoped to
// the nav region because the Welcome window duplicates them as quick links.
async function enterOs(page: Page) {
  await page.goto("/");
  await page.getByRole("button", { name: "Enter NIKHIL//OS" }).click();
  await expect(page.locator(".desktop-shell")).toBeVisible({ timeout: 15_000 });
  return page.getByRole("navigation", { name: "Applications" });
}

async function openApp(nav: ReturnType<Page["getByRole"]>, name: string) {
  await nav.getByRole("button", { name }).click();
}

test("boots into the desktop", async ({ page }) => {
  await page.goto("/");

  await expect(page.getByRole("heading", { level: 1 })).toContainText("NIKHIL");
  await page.getByRole("button", { name: "Enter NIKHIL//OS" }).click();

  // Real boot sequence: subsystem lines are revealed, then the desktop mounts.
  await expect(page.locator(".boot-console")).toBeVisible();
  await expect(page.locator(".desktop-shell")).toBeVisible({ timeout: 15_000 });

  await expect(page.locator(".sysbar")).toBeVisible();
  await expect(page.getByRole("navigation", { name: "Applications" })).toBeVisible();
});

test("terminal runs a real pipeline against the live core", async ({ page }) => {
  const nav = await enterOs(page);
  await openApp(nav, "Terminal");

  const shell = page.getByLabel("shell input");
  await expect(shell).toBeVisible();

  // `ps | grep ai` executes through the real parser + pipeline executor.
  await shell.fill("ps | grep ai");
  await shell.press("Enter");
  await expect(page.locator(".terminal-scroll")).toContainText("ai-core", {
    timeout: 5_000,
  });

  // The filesystem is real: /proc is a live virtual mount. uptime(1) prints
  // "<tick>.00 0.00" — a wall-clock counter driven by the running kernel.
  await shell.fill("cat /proc/uptime");
  await shell.press("Enter");
  await expect(page.locator(".terminal-scroll")).toContainText(/\d+\.00 0\.00/, {
    timeout: 5_000,
  });
});

test("system monitor shows real telemetry from the core", async ({ page }) => {
  const nav = await enterOs(page);
  await openApp(nav, "System Monitor");

  await expect(page.getByText(/Processes/)).toBeVisible();
  // The scheduler is real: a process is either running or ready.
  await expect(page.locator(".data-table tbody tr").first()).toContainText(
    /running|ready|new/i,
  );
});

test("profile apps render knowledge data", async ({ page }) => {
  const nav = await enterOs(page);
  await openApp(nav, "Projects");

  // The project card renders the embedded profile.json, not a placeholder.
  const projectsWin = page.locator('.os-window[aria-label="Projects"]');
  await expect(projectsWin).toContainText("NIKHIL//OS");
  await expect(projectsWin).toContainText(
    "An AI-native personal computing environment",
  );
});

test("resume app renders experience and education", async ({ page }) => {
  const nav = await enterOs(page);
  await openApp(nav, "Resume");

  const resumeWin = page.locator('.os-window[aria-label="Resume"]');
  await expect(resumeWin).toContainText("WhatBytes");
  await expect(resumeWin).toContainText("Infosys Springboard");
  await expect(resumeWin).toContainText("KL University");
});
