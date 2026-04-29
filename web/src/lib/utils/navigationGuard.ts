export function getNavigationTarget(url: URL | null | undefined): string | null {
	if (!url) return null;
	return `${url.pathname}${url.search}${url.hash}`;
}

export function shouldBlockNavigation(
	isDirty: boolean,
	bypassNavigationGuard: boolean,
	currentUrl: URL | null | undefined,
	nextUrl: URL | null | undefined
): boolean {
	if (!isDirty || bypassNavigationGuard || !nextUrl) return false;
	return getNavigationTarget(currentUrl) !== getNavigationTarget(nextUrl);
}
