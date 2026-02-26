import { sharedTransform, SHARED_CONSTANT } from "./shared.mjs";

it("should produce correct runtime results", async () => {
	expect(sharedTransform("HELLO")).toBe("hello");
	expect(SHARED_CONSTANT).toBe(42);

	const page = await import(/* webpackChunkName: "page" */ "./page.mjs");
	expect(page.renderPage("hello")).toBe("hello_HELLO_42[hello]");

	const page2 = await import(/* webpackChunkName: "page2" */ "./page2.mjs");
	expect(page2.renderPage2("world")).toBe("world_WORLD_page2");
});

it("should concatenate modules from parent chunks into async chunks", () => {
	const fs = __non_webpack_require__("fs");
	const path = __non_webpack_require__("path");

	const pageChunkFiles = fs.readdirSync(__dirname).filter(f => f.startsWith("page") && f.endsWith(".js") && !f.startsWith("page2"));
	const page2ChunkFiles = fs.readdirSync(__dirname).filter(f => f.startsWith("page2") && f.endsWith(".js"));

	expect(pageChunkFiles.length).toBeGreaterThan(0);
	expect(page2ChunkFiles.length).toBeGreaterThan(0);

	const pageContent = fs.readFileSync(path.join(__dirname, pageChunkFiles[0]), "utf-8");
	const page2Content = fs.readFileSync(path.join(__dirname, page2ChunkFiles[0]), "utf-8");

	// page_only_helper.mjs is still concatenated (same chunk)
	expect(pageContent).toContain("CONCATENATED MODULE");

	// With cross-chunk concatenation, helpers.mjs from the parent common chunk
	// should also be concatenated into both page chunks
	expect(page2Content).toContain("CONCATENATED MODULE");
});
