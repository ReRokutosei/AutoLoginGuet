#!/bin/bash

# 配置你的参数
USERNAME=""
PASSWORD=""
ISP="" # 运营商: ""(校园网), "@cmcc"(中国移动), "@unicom"(中国联通), "@telecom"(中国电信), "@glgd"(中国广电)
LOGIN_IP="http://10.0.1.5"

FULL_USERNAME="${USERNAME}${ISP}"
PARAMS="callback=dr1003&DDDDD=${FULL_USERNAME}&upass=${PASSWORD}&0MKKey=123456"

RESPONSE=$(curl -s -X POST "${LOGIN_IP}/drcom/login" -d "${PARAMS}" 2>/dev/null)

if [ $? -ne 0 ]; then
    NOTIFICATION="登录失败！网络请求错误"

    if command -v osascript >/dev/null 2>&1; then
        osascript -e "display notification \"${NOTIFICATION}\""
    elif command -v notify-send >/dev/null 2>&1; then
        notify-send "${NOTIFICATION}"
    else
        echo "${NOTIFICATION}"
    fi
    exit 1
fi

if [[ $RESPONSE == *"注销页"* ]] || [[ $RESPONSE == *"认证成功页"* ]] || [[ $RESPONSE == *"Dr.COMWebLoginID_3.htm"* ]] || [[ $RESPONSE == *"\"result\":1"* ]]; then
    NOTIFICATION="登录成功！"

    if command -v osascript >/dev/null 2>&1; then
        osascript -e "display notification \"${NOTIFICATION}\""
    elif command -v notify-send >/dev/null 2>&1; then
        notify-send "${NOTIFICATION}"
    else
        echo "${NOTIFICATION}"
    fi
elif [[ $RESPONSE == *"msga='ldap auth error'"* ]] || [[ $RESPONSE == *"ldap auth error"* ]] || [[ $RESPONSE == *"Msg=01"* ]]; then
    NOTIFICATION="登录失败！请检查配置信息"

    if command -v osascript >/dev/null 2>&1; then
        osascript -e "display notification \"${NOTIFICATION}\""
    elif command -v notify-send >/dev/null 2>&1; then
        notify-send "${NOTIFICATION}"
    else
        echo "${NOTIFICATION}"
    fi
else
    NOTIFICATION="登录异常！请检查代理或硬件"

    if command -v osascript >/dev/null 2>&1; then
        osascript -e "display notification \"${NOTIFICATION}\""
    elif command -v notify-send >/dev/null 2>&1; then
        notify-send "${NOTIFICATION}"
    else
        echo "${NOTIFICATION}"
    fi
fi
