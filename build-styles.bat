@echo off
echo 正在构建CSS样式文件...
npx tailwindcss -i ./src/style.css -o ./src/style.css --minify
echo ✅ CSS构建完成！