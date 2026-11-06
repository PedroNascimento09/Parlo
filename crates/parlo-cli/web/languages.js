function writeTargets(pairs, names, from) {
  const codes = new Set();
  for (const [source, target] of pairs) {
    if (!from || source === from) {
      codes.add(target);
    }
  }
  return [...codes].sort((a, b) => (names[a] || a).localeCompare(names[b] || b));
}
