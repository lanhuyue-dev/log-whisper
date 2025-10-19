// Test Docker plugin chain detection
const fs = require('fs');
const path = require('path');

// Read the container log file
const testFile = path.join('C:', 'lanhuyue', 'internal', 'log-whisper', 'data', 'container-log.txt');
const logContent = fs.readFileSync(testFile, 'utf8');

console.log('=== Docker Plugin Chain Detection Test ===');
console.log('Test file:', testFile);
console.log('Content length:', logContent.length);
console.log('Number of lines:', logContent.split('\n').length);

// Test Docker JSON detection logic
const lines = logContent.split('\n').filter(line => line.trim());
console.log('\n=== Docker JSON Detection Test ===');

// Check first few lines for Docker JSON pattern
const sampleLines = lines.slice(0, 5);
sampleLines.forEach((line, index) => {
    console.log(`\n--- Line ${index + 1} ---`);
    console.log('Raw line:', line);

    const trimmed = line.trimStart();
    const startsWithBrace = trimmed.startsWith('{');
    const containsLogField = trimmed.includes('"log"');
    const containsStreamField = trimmed.includes('"stream"');

    console.log(`Starts with '{': ${startsWithBrace}`);
    console.log(`Contains "log": ${containsLogField}`);
    console.log(`Contains "stream": ${containsStreamField}`);

    const isDockerJson = startsWithBrace && (containsLogField || containsStreamField);
    console.log(`Is Docker JSON: ${isDockerJson}`);

    if (isDockerJson) {
        try {
            const json = JSON.parse(line);
            console.log('✅ Valid JSON detected');
            console.log('  log field exists:', !!json.log);
            console.log('  stream field exists:', !!json.stream);
            console.log('  time field exists:', !!json.time);

            if (json.log) {
                // Check if log content contains Java GC pattern
                const gcLogPattern = /^\[[^\]]+\]\[([^\]]+)\]/;
                const gcMatch = json.log.match(gcLogPattern);
                if (gcMatch) {
                    console.log('  🎯 Contains Java GC log with level:', gcMatch[1]);
                    const normalizedLevel = gcMatch[1].trim().replace(/warning/, 'WARN')
                                                            .replace(/info/, 'INFO')
                                                            .replace(/error/, 'ERROR')
                                                            .replace(/debug/, 'DEBUG')
                                                            .replace(/trace/, 'DEBUG')
                                                            .toUpperCase();
                    console.log('  📝 Normalized level:', normalizedLevel);
                }
            }
        } catch (e) {
            console.log('❌ Invalid JSON:', e.message);
        }
    }
});

console.log('\n=== Detection Summary ===');
const dockerJsonLines = lines.filter(line => {
    const trimmed = line.trimStart();
    return trimmed.startsWith('{') && (trimmed.includes('"log"') || trimmed.includes('"stream"'));
});

console.log(`Total lines: ${lines.length}`);
console.log(`Docker JSON lines: ${dockerJsonLines.length}`);
console.log(`Percentage: ${(dockerJsonLines.length / lines.length * 100).toFixed(2)}%`);

if (dockerJsonLines.length > lines.length * 0.8) {
    console.log('✅ This file should be detected as Docker JSON format');
    console.log('✅ Docker plugin chain should be selected');
} else {
    console.log('❌ This file may not be detected as Docker JSON format');
}

console.log('\n=== Expected Behavior ===');
console.log('1. Enhanced plugin manager should detect Docker JSON format');
console.log('2. Docker plugin chain should be selected');
console.log('3. DockerJsonFilter should parse JSON and extract log content');
console.log('4. Java GC log levels should be mapped to system levels');
console.log('5. Formatted content should show clean messages without JSON');