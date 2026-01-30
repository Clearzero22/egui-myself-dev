#!/bin/bash

echo "========================================"
echo "GNOME Wayland 双击标题栏诊断工具"
echo "========================================"
echo ""

echo "1. 检查当前环境："
echo "   DESKTOP_SESSION: $DESKTOP_SESSION"
echo "   XDG_SESSION_TYPE: $XDG_SESSION_TYPE"
echo "   XDG_CURRENT_DESKTOP: $XDG_CURRENT_DESKTOP"
echo ""

echo "2. 检查双击标题栏设置："
gsettings get org.gnome.desktop.wm.preferences action-double-click-titlebar
echo ""

echo "3. 检查按钮布局："
gsettings get org.gnome.desktop.wm.preferences button-layout
echo ""

echo "4. 检查是否使用客户端端装饰 (CSD)："
echo "   GNOME 默认使用 CSD（客户端端装饰）"
echo "   某些应用可能不支持 CSD 协议"
echo ""

echo "5. 常见问题："
echo "   ❌ 如果应用不支持 GNOME 的 CSD 协议"
echo "      → 双击标题栏可能不起作用"
echo ""
echo "   ❌ 如果应用使用了自定义窗口装饰"
echo "      → 需要应用自己实现双击功能"
echo ""

echo "6. egui 应用在 Wayland 上的限制："
echo "   • egui 使用 winit 库创建窗口"
echo "   • winit 的 Wayland 支持可能在某些方面不完整"
echo "   • 系统标题栏的双击功能依赖 Wayland 协议实现"
echo ""

echo "7. 测试其他应用："
echo "   尝试双击以下应用的标题栏："
echo "   • Files (Nautilus)"
echo "   • GNOME Terminal"
echo "   • Settings"
echo "   如果这些应用的双击起作用，说明系统设置正确"
echo ""

echo "========================================"
echo "建议的解决方案："
echo "========================================"
echo ""
echo "方案 A: 使用 F11 快捷键（已实现 ✅）"
echo "   - 在示例程序中已添加 F11 全屏切换"
echo "   - 跨平台一致，不受窗口管理器影响"
echo ""

echo "方案 B: 尝试切换到 X11 会话"
echo "   - 退出当前会话"
echo "   - 在登录界面点击齿轮图标"
echo "   - 选择 'GNOME on X11'"
echo "   - X11 的窗口管理更成熟稳定"
echo ""

echo "方案 C: 检查 GNOME Shell 扩展"
echo "   - 某些扩展可能干扰标题栏行为"
echo "   - 运行 'gnome-extensions list' 查看已安装扩展"
echo "   - 尝试禁用所有扩展后测试"
echo ""

echo "方案 D: 使用自定义窗口标题栏"
echo "   - 移除系统标题栏 (with_decorations(false))"
echo "   - 自己实现双击最大化功能"
echo "   - 参考 examples/custom_window_frame"
echo ""
