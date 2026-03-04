export const a = 1;
export const b = "hello";
export function c() {
	return 42;
}

it("should export values correctly with per-export define", () => {
	expect(a).toBe(1);
	expect(b).toBe("hello");
	expect(c()).toBe(42);
});
