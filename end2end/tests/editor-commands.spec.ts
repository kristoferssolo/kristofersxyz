import { expect, type Page, test } from "@playwright/test";

async function runHelpCommand(page: Page) {
  await page.locator("main").click();
  await page.keyboard.type(":");
  await expect(page.locator("footer > div").nth(1)).toHaveCSS(
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

test("commands work on the homepage and project details", async ({
  page,
}) => {
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

test("desktop status starts beyond the native link preview area", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto("http://localhost:3000/");

  const sidebar = await page
    .getByRole("navigation", { name: "Portfolio" })
    .boundingBox();
  const status = await page.locator("footer > div").nth(1).boundingBox();

  expect(sidebar).not.toBeNull();
  expect(status).not.toBeNull();
  if (sidebar === null || status === null) {
    throw new Error("the shared page chrome did not render");
  }

  expect(status.x).toBeGreaterThanOrEqual(sidebar.x + sidebar.width - 1);
});
