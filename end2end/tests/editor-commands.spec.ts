import { expect, type Page, test } from "@playwright/test";

async function runHelpCommand(page: Page) {
  await page.locator("main").click();
  await page.keyboard.type(":");
  await expect(page.locator("footer > div").first()).toHaveCSS(
    "border-radius",
    "0px",
  );
  await page.keyboard.type("help");
  await expect(page.locator("footer")).toContainText(":help");
  await page.keyboard.press("Enter");
  await expect(
    page.getByRole("dialog", { name: "Keyboard help" }),
  ).toBeVisible();
}

test("commands work on the homepage and project details", async ({ page }) => {
  await page.goto("http://localhost:3000/");
  await runHelpCommand(page);

  await page.goto("http://localhost:3000/work/guenther");
  await runHelpCommand(page);

  await page.keyboard.press("Escape");
  await page.keyboard.type(":contact");
  await expect(page.locator("footer")).toContainText(":contact");
  await page.keyboard.press("Enter");
  await expect(page).toHaveURL("http://localhost:3000/#contact");
});

test("the status line spans the viewport, its text clear of the link preview", async ({
  page,
}) => {
  const width = 1280;
  await page.setViewportSize({ width, height: 800 });
  await page.goto("http://localhost:3000/");

  const status = await page.locator("footer > div").first().boundingBox();
  // Browsers draw hovered-link destinations over the lower left, so the
  // filename lives in the right cluster and only the mode block is exposed.
  const filename = await page
    .locator("footer")
    .getByText("kristofers.xyz")
    .boundingBox();

  expect(status).not.toBeNull();
  expect(filename).not.toBeNull();
  if (status === null || filename === null) {
    throw new Error("the shared page chrome did not render");
  }

  expect(status.x).toBeLessThanOrEqual(1);
  expect(status.width).toBeGreaterThanOrEqual(width - 1);
  expect(filename.x).toBeGreaterThan(width / 2);
});

test("ctrl+b collapses the sidebar and movement leaves it collapsed", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto("http://localhost:3000/");

  const navigation = page.getByRole("navigation", { name: "Portfolio" });
  const toggle = page.getByRole("button", { name: /portfolio navigation/ });

  await expect(navigation).toBeVisible();
  await expect(toggle).toHaveAttribute("aria-expanded", "true");

  // The toggle is the first tab stop, and the editor leaves its Enter alone.
  await page.keyboard.press("Tab");
  await expect(toggle).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(navigation).toBeHidden();
  await expect(toggle).toHaveAttribute("aria-expanded", "false");

  // G jumps to contact, which the homepage selects in place. Movement must
  // leave the layout exactly as the reader set it.
  await page.keyboard.press("G");
  await expect(page.locator("footer")).toContainText("[5/5]");
  await expect(navigation).toBeHidden();

  await toggle.click();
  await expect(navigation).toBeVisible();
});

test(":e opens a page by name from another route", async ({ page }) => {
  await page.goto("http://localhost:3000/work/guenther");
  await page.locator("main").click();

  await page.keyboard.type(":e traxor");
  await expect(page.locator("footer")).toContainText(":e traxor");
  await page.keyboard.press("Enter");
  await expect(page).toHaveURL("http://localhost:3000/work/traxor");

  // A name no entry answers to leaves the reader where they were.
  await page.keyboard.type(":e nowhere");
  await page.keyboard.press("Enter");
  await expect(page.locator('[aria-live="polite"]')).toContainText(
    "E94: No matching buffer for nowhere",
  );
  await expect(page).toHaveURL("http://localhost:3000/work/traxor");
});
