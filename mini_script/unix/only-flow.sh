#!/bin/bash

# 配置你的参数
USERNAME=""
PASSWORD=""


show_notification() {
    local message="$1"
    if command -v osascript >/dev/null 2>&1; then
        osascript -e "display notification \"$message\""
    elif command -v notify-send >/dev/null 2>&1; then
        notify-send "$message"
    else
        echo "$message"
    fi
}

md5_hash() {
    echo -n "$1" | md5sum | cut -d ' ' -f1
}

get_login_page() {
    curl -s -c cookies.txt "https://nicdrcom.guet.edu.cn/Self/login/" 2>/dev/null
}

get_checkcode() {
    echo "$1" | grep 'name="checkcode"' | sed -n 's/.*value="\([^"]*\)".*/\1/p'
}

get_captcha() {
    local timestamp=$(date +%s%3N)
    curl -s -b cookies.txt -c cookies.txt "https://nicdrcom.guet.edu.cn/Self/login/randomCode?t=$timestamp" -o /dev/null 2>/dev/null
}

site_login() {
    local username="$1"
    local password="$2"
    local checkcode="$3"
    
    local random_code=$(printf "%04d" $((RANDOM % 10000)))
    local encrypted_password=$(md5_hash "$password")
    
    local response=$(curl -s -b cookies.txt -c cookies.txt -X POST "https://nicdrcom.guet.edu.cn/Self/login/verify" \
        -H "Content-Type: application/x-www-form-urlencoded" \
        -d "account=$username&password=$encrypted_password&checkcode=$checkcode&code=$random_code" 2>/dev/null)
    
    local dashboard=$(curl -s -b cookies.txt -c cookies.txt "https://nicdrcom.guet.edu.cn/Self/dashboard" 2>/dev/null)
    
    if echo "$dashboard" | grep -q "leftFlow"; then
        echo "$dashboard"
        return 0
    else
        return 1
    fi
}

get_remaining_flow() {
    local dashboard_html="$1"
    
    if echo "$dashboard_html" | grep -qE '<dt>\s*[0-9]+(\.[0-9]+)?\s*<small[^>]*>M</small>\s*</dt>\s*<dd>剩余流量</dd>'; then
        local flow=$(echo "$dashboard_html" | grep -oE '<dt>\s*[0-9]+(\.[0-9]+)?\s*<small[^>]*>M</small>\s*</dt>\s*<dd>剩余流量</dd>' | grep -oE '[0-9]+(\.[0-9]+)?')
        echo "$flow"
        return 0
    fi
    
    if echo "$dashboard_html" | grep -q '"leftFlow":'; then
        local flow=$(echo "$dashboard_html" | grep -o '"leftFlow":[0-9]*\.?[0-9]*' | cut -d: -f2)
        echo "$flow"
        return 0
    fi
    
    return 1
}

format_flow_info() {
    local left_flow="$1"
    
    case $left_flow in
        0.0)
            echo "校园网 流量耗尽，限速不限量生效"
            ;;
        *)
            if (( $(echo "$left_flow < 1024.0" | bc -l) )); then
                printf "校园网 剩余流量 %.2f MB" "$left_flow"
            else
                printf "校园网 剩余流量 %.2f GB" "$(echo "$left_flow / 1024.0" | bc -l)"
            fi
            ;;
    esac
}

main() {
    local login_page=$(get_login_page)
    if [ -z "$login_page" ]; then
        show_notification "获取登录页面失败"
        rm -f cookies.txt
        exit 1
    fi
    
    local checkcode=$(get_checkcode "$login_page")
    if [ -z "$checkcode" ]; then
        show_notification "提取checkcode失败"
        rm -f cookies.txt
        exit 1
    fi
    
    get_captcha
    
    local dashboard_content
    if ! dashboard_content=$(site_login "$USERNAME" "$PASSWORD" "$checkcode"); then
        show_notification "登录失败"
        rm -f cookies.txt
        exit 1
    fi
    
    local left_flow
    if ! left_flow=$(get_remaining_flow "$dashboard_content"); then
        show_notification "获取流量信息失败"
        rm -f cookies.txt
        exit 1
    fi
    
    local flow_info=$(format_flow_info "$left_flow")
    show_notification "$flow_info"
    
    rm -f cookies.txt
}

main "$@"