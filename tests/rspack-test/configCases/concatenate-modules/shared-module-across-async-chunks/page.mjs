import { sharedTransform } from "./shared.mjs";
import { formatValue, doubleValue } from "./helpers.mjs";
import { localFormat } from "./page_only_helper.mjs";

export const renderPage = (value) => {
	return sharedTransform(value) + "_" + formatValue(value) + "_" + doubleValue(21) + localFormat(value);
};
