// Quick test of container.log parsing through direct invoke
const fs = require('fs');
const path = require('path');

// Read the container log file
const testFile = path.join('C:', 'lanhuyue', 'internal', 'log-whisper', 'data', 'container-log.txt');
const logContent = fs.readFileSync(testFile, 'utf8');

console.log('=== Container Log Parsing Test ===');
console.log('File:', testFile);
console.log('Size:', logContent.length, 'bytes');

// Take first few lines for testing
const lines = logContent.split('\n').filter(line => line.trim()).slice(0, 3);

console.log('\n=== Test Data (first 3 lines) ===');
lines.forEach((line, i) => {
    console.log(`${i + 1}: ${line.substring(0, 100)}...`);
});

console.log('\n=== Expected Docker Chain Processing ===');
console.log('1. Chain Selection: Docker (100% JSON format)');
console.log('2. DockerJsonFilter: Parse JSON, extract log content');
console.log('3. Java GC Log Level Mapping: warning->WARN, info->INFO');
console.log('4. SpringBootFilter: Process any SpringBoot format');
console.log('5. ContentEnhancer: Add enhancements');
console.log('6. JsonStructureFilter: Final JSON output');
console.log('7. Expected Result: Clean messages without JSON wrapper');

console.log('\n=== Ready for UI Test ===');
console.log('Application should show clean log entries like:');
console.log('- WARN -XX:+PrintGCDetails is deprecated...');
console.log('- INFO CardTable entry size: 512');
console.log('- INFO Using G1');