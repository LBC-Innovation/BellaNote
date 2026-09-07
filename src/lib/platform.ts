export function isMac(): boolean {
  return /Mac/i.test(navigator.userAgent) && !/iPhone|iPad/i.test(navigator.userAgent);
}

export function isWindows(): boolean {
  return /Windows/i.test(navigator.userAgent);
}
