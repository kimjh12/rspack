import { sharedTransform } from "./shared.mjs";
import { formatValue } from "./helpers.mjs";

export const renderPage2 = (value) => {
	return sharedTransform(value) + "_" + formatValue(value) + "_page2";
};
