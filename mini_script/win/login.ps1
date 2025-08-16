$Username = ""
$Password = ""
$ISP = ""  # 运营商: ""(校园网), "@cmcc"(中国移动), "@unicom"(中国联通), "@telecom"(中国电信)
$LoginIP = "http://10.0.1.5"
$FullUsername = $Username + $ISP
$Body = "callback=dr1003&DDDDD=$FullUsername&upass=$Password&0MKKey=123456"

Import-Module BurntToast

try {
    $Response = Invoke-WebRequest -Uri "$LoginIP/drcom/login" -Method POST -Body $Body -ContentType "application/x-www-form-urlencoded" -ErrorAction Stop
    if ($Response.Content -match "注销页|认证成功页|Dr.COMWebLoginID_3.htm|`"result`":1") {
        try {
            New-BurntToastNotification -Text "登录成功！已接入广域网" -Applogo None
        } catch {
            Write-Host "登录成功：已接入广域网"
        }
    } elseif ($Response.Content -match "msga='ldap auth error'|ldap auth error|Msg=01") {
        try {
            New-BurntToastNotification -Text "登录失败！请检查配置信息" -Applogo None
        } catch {
            Write-Host "登录失败！请检查配置信息"
        }
    } else {
        try {
            New-BurntToastNotification -Text "登录异常！请检查代理或硬件" -Applogo None
        } catch {
            Write-Host "登录异常！请检查代理或硬件"
        }
    }
} catch {
    try {
        New-BurntToastNotification -Text "登录失败！网络请求错误" -Applogo None
    } catch {
        Write-Host "登录失败！网络请求错误"
    }
}