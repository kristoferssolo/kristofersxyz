import { expect, test } from "@playwright/test";

test("homepage title", async ({ page }) => {
  await page.goto("http://localhost:3000/");

  await expect(page).toHaveTitle("Kristofers Solo");
});

test("hero identity", async ({ page }) => {
  await page.goto("http://localhost:3000/");

  await expect(page.locator("h1")).toContainText("Kristofers Solo");
});

test("public links", async ({ page }) => {
  await page.goto("http://localhost:3000/");

  await expect(page.getByRole("link", { name: "Codeberg" }).first()).toHaveAttribute(
    "href",
    "https://codeberg.org/kristoferssolo",
  );
  await expect(page.getByRole("link", { name: "GitHub" }).first()).toHaveAttribute(
    "href",
    "https://github.com/kristoferssolo",
  );
  await expect(page.getByRole("link", { name: "Mastodon" }).first()).toHaveAttribute(
    "href",
    "https://fosstodon.org/@kristofers_solo",
  );
  await expect(page.getByRole("link", { name: "Email" }).first()).toHaveAttribute(
    "href",
    "mailto:dev@kristofers.xyz",
  );
});

test("projects section", async ({ page }) => {
  await page.goto("http://localhost:3000/");

  await expect(page.getByRole("heading", { name: "Selected Work" })).toBeVisible();
  await expect(page.locator(".project-card").filter({ hasText: "kristofers.xyz" })).toHaveCount(1);
});
