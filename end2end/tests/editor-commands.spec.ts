import { expect, type Page, test } from "@playwright/test";

async function runHelpCommand(page: Page) {
  await page.locator("main").click();
  await page.keyboard.type(":help");
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
