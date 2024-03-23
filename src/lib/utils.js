export function getFirstLine(str, max_chars = 25) {
    const newlineIndex = str.indexOf("\n");
    const firstLine = newlineIndex !== -1 ? str.slice(0, newlineIndex) : str;
    return firstLine.slice(0, max_chars);
}