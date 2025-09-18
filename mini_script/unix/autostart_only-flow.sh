#!/bin/bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCRIPT_NAME="only-flow.sh"
SERVICE_NAME="autologin-guet-flow.service"
PLIST_NAME="com.autologin.guet.flow.plist"

echo "设置流量查询脚本自启动..."

if [ ! -f "$SCRIPT_DIR/$SCRIPT_NAME" ]; then
    echo "错误: $SCRIPT_NAME 文件未找到"
    exit 1
fi

if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "检测到 MacOS"
    
    PLIST_PATH="$HOME/Library/LaunchAgents/$PLIST_NAME"
    cat > "$PLIST_PATH" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.autologin.guet.flow</string>
    <key>ProgramArguments</key>
    <array>
        <string>/bin/bash</string>
        <string>$SCRIPT_DIR/$SCRIPT_NAME</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>LaunchOnlyOnce</key>
    <true/>
</dict>
</plist>
EOF
    
    launchctl load "$PLIST_PATH"
    echo "已创建并加载 plist 文件: $PLIST_PATH"
    echo "如需卸载，请运行: launchctl unload $PLIST_PATH"

elif command -v systemctl &> /dev/null; then
    echo "检测到 Linux"
    
    SERVICE_PATH="$HOME/.config/systemd/user/$SERVICE_NAME"
    mkdir -p "$(dirname "$SERVICE_PATH")"
    
    cat > "$SERVICE_PATH" << EOF
[Unit]
Description=Auto Login GUET Flow Query
After=network.target

[Service]
Type=oneshot
ExecStart=/bin/bash $SCRIPT_DIR/$SCRIPT_NAME
RemainAfterExit

[Install]
WantedBy=default.target
EOF
    
    systemctl --user daemon-reload
    systemctl --user enable "$SERVICE_NAME"
    systemctl --user start "$SERVICE_NAME"
    
    echo "已创建并启动 systemd 服务: $SERVICE_PATH"
    echo "服务状态: $(systemctl --user is-active "$SERVICE_NAME")"
    echo "如需停止服务，请运行: systemctl --user stop $SERVICE_NAME"
    echo "如需禁用服务，请运行: systemctl --user disable $SERVICE_NAME"

else
    echo "不支持的操作系统"
    exit 1
fi

echo "设置完成"