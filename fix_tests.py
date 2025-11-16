import re

with open('tests/operations_tests.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Add symbol_config: None after each convolutional_config line
content = re.sub(
    r'(\s+convolutional_config: (?:None|config|interleave_config|deinterleave_config),)\n(\s+\})',
    r'\1\n            symbol_config: None,\n\2',
    content
)

with open('tests/operations_tests.rs', 'w', encoding='utf-8') as f:
    f.write(content)

print("Updated operations_tests.rs with symbol_config fields")
