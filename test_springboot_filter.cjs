// Test SpringBootFilter regex with test file format
const testLines = [
    '2025-01-15 10:30:45.123 INFO  [main] Starting application...',
    '2025-01-15 10:30:46.789 WARN  [network] Connection timeout after 5000ms',
    '2025-01-15 10:30:47.012 ERROR [database] Failed to execute query: SELECT * FROM users WHERE id = 12345',
    '2025-01-15 10:30:48.678 INFO  [api] GET /api/users/12345 - 200 OK (15ms)',
    '2025-01-15 10:30:49.901 WARN  [security] Suspicious login attempt from IP 192.168.1.100'
];

console.log('=== SpringBootFilter Regex Test ===');

// The updated regex pattern
const regex = /^(\d{4}-\d{2}-\d{2}[T\s]\d{2}:\d{2}:\d{2}[.,]\d{3}(?:Z)?)\s+([A-Z]+)\s+(?:\d+\s+---\s+)?\[\s*([^\]]+)\s*\](?:\s+([^\s:]+)\s*:\s*)?(.*)$/;

console.log('Regex pattern:', regex);
console.log('');

testLines.forEach((line, index) => {
    console.log(`--- Test Line ${index + 1} ---`);
    console.log('Original:', line);

    const match = line.match(regex);
    if (match) {
        console.log('✅ Match successful!');
        console.log('  Timestamp:', match[1]);
        console.log('  Level:', match[2]);
        console.log('  Thread:', match[3]);
        console.log('  Logger (optional):', match[4] || '(none)');
        console.log('  Message:', match[5]);
        console.log('  Clean message (no level prefix):', match[5]);
    } else {
        console.log('❌ No match');
    }
    console.log('');
});

console.log('=== Expected Results ===');
console.log('If regex works correctly, the message should NOT contain the log level prefix.');
console.log('This will fix the duplicate display issue in the UI.');