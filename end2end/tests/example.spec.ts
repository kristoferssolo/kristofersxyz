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

  await expect(page.getByRole("link", { name: "Codeberg" })).toHaveAttribute(
    "href",
    "https://codeberg.org/kristoferssolo",
  );
  await expect(page.getByRole("link", { name: "GitHub" })).toHaveAttribute(
    "href",
    "https://github.com/kristoferssolo",
  );
  await expect(page.getByRole("link", { name: "Mastodon" })).toHaveAttribute(
    "href",
    "https://fosstodon.org/@kristofers_solo",
  );
  await expect(page.getByRole("link", { name: "Email" })).toHaveAttribute(
    "href",
    "mailto:dev@kristofers.xyz",
  );
});

test("projects section", async ({ page }) => {
  await page.goto("http://localhost:3000/");

  await expect(page.getByRole("heading", { name: "Selected Work" })).toBeVisible();
  await expect(page.locator("article").filter({ hasText: "kristofers.xyz" })).toHaveCount(1);
});

test("the sign-in throttle counts down without another render", async ({ page }) => {
  await page.route("**/api/login", async (route) => {
    await route.fulfill({
      status: 429,
      contentType: "application/json",
      headers: {
        "retry-after": "3",
        serverfnerror: "/api/login",
      },
      body: JSON.stringify({
        TooManyAttempts: { retry_after_seconds: 3 },
      }),
    });
  });
  await page.goto("http://localhost:3000/login");

  await page.getByLabel("Username").fill("owner");
  await page.getByLabel("Password").fill("wrong password");
  await page.getByRole("button", { name: "Sign in" }).click();

  const error = page.getByText("Too many sign-in attempts", { exact: false });
  await expect(error).toContainText("3 seconds");
  await expect(error).toContainText("2 seconds", { timeout: 2_000 });
  await expect(error).toContainText("0 seconds", { timeout: 3_000 });
});
