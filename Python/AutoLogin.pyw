# 标准库导入
import os
import time
import random
import logging
from datetime import datetime, timedelta

# 第三方库导入
import yaml
import requests
from plyer import notification

# 全局变量
start_time = time.time()
base_path = os.path.dirname(os.path.abspath(__file__))  # 获取当前脚本所在目录
config_path = os.path.join(base_path, 'config.yaml')

def load_config():
    try:
        with open(config_path, 'r', encoding='utf-8') as f:
            config = yaml.safe_load(f)
            # 处理相对路径
            if config['notification']['use_custom_icon']:
                icon_path = config['notification']['icon_path']
                if icon_path.startswith('./') or icon_path.startswith('../'):
                    config['notification']['icon_path'] = os.path.abspath(
                        os.path.join(base_path, icon_path)
                    )
            # 处理日志路径
            if config['logging']['enable_logging']:
                log_path = config['logging']['log_file_path']
                if log_path.startswith('./') or log_path.startswith('../'):
                    config['logging']['log_file_path'] = os.path.join(
                        base_path, log_path.lstrip('./').lstrip('../')
                    )
            return config
    except Exception as e:
        raise Exception(f"无法加载配置文件: {str(e)}")

class LogManager:
    def __init__(self, config):
        self.config = config
        if config['logging']['enable_logging']:
            logging.basicConfig(
                filename=config['logging']['log_file_path'],
                level=logging.INFO,
                format='%(asctime)s - %(levelname)s - %(message)s'
            )

    def clean_old_logs(self):
        if not self.config['logging']['enable_logging']:
            return
            
        if not os.path.exists(self.config['logging']['log_file_path']):
            return
        
        retention_days = self.config['logging']['info_log_retention_days']
        cutoff_date = datetime.now() - timedelta(days=retention_days)
        
        with open(self.config['logging']['log_file_path'], 'r') as f:
            lines = f.readlines()
        
        new_lines = []
        for line in lines:
            try:
                log_date = datetime.strptime(line[:19], '%Y-%m-%d %H:%M:%S')
                if 'INFO' in line and log_date < cutoff_date:
                    continue
                new_lines.append(line)
            except:
                new_lines.append(line)
        
        with open(self.config['logging']['log_file_path'], 'w') as f:
            f.writelines(new_lines)

    def log_event(self, level, message):
        if not self.config['logging']['enable_logging']:
            return
            
        if level == 'ERROR':
            logging.error(message)
        elif level == 'WARNING':
            logging.warning(message)
        else:
            logging.info(message)

# 更新配置
config = load_config()
log_manager = LogManager(config)

# 清理旧日志
log_manager.clean_old_logs()

# 显示通知
def show_notification(title, message):
    try:
        if config['notification']['use_custom_icon']:
            notification.notify(
                title=title,
                message=message,
                app_icon=config['notification']['icon_path']
            )
        else:
            notification.notify(
                title=title,
                message=message
            )
    except Exception as e:
        log_manager.log_event('ERROR', f'Notification failed: {str(e)}')

time.sleep(random.uniform(0, 5))  # 随机延迟 0 到 5 秒，等待系统稳定

# 添加在全局配置后面
headers = {
    'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
    'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9',
    'Accept-Language': 'zh-CN,zh;q=0.9,en;q=0.8',
    'Connection': 'keep-alive',
    'Referer': 'http://10.0.1.5/',
}

try:
    # 先检查登录状态
    try:
        session = requests.Session()
        r = session.get(config['network']['login_ip'], headers=headers, timeout=10)
        req = r.text
        elapsed_time = time.time() - start_time

        if config['network']['signed_in_title'] in req:
            log_message = f'Device already logged in. Elapsed time: {elapsed_time:.2f} seconds.'
            login_status = f"该设备已经登录，本次用时 {elapsed_time:.2f} 秒"
            log_manager.log_event('INFO', log_message)
            show_notification("校园网状态", login_status)
            
        elif config['network']['not_sign_in_title'] in req:
            # 尝试登录
            r = session.get(
                config['network']['sign_parameter'], 
                headers=headers,
                timeout=10
            )
            req = r.text
            elapsed_time = time.time() - start_time
            
            if config['network']['result_return'] in req:
                log_message = f'Login successful. Elapsed time: {elapsed_time:.2f} seconds.'
                login_status = f"登录成功，本次用时 {elapsed_time:.2f} 秒"
                log_manager.log_event('INFO', log_message)
            else:
                log_message = f'Login failed. Elapsed time: {elapsed_time:.2f} seconds.'
                login_status = f"登录失败，本次用时 {elapsed_time:.2f} 秒"
                log_manager.log_event('WARNING', log_message)
            
            show_notification("校园网状态", login_status)
            
        else:
            log_message = f'Not connected to campus network. Elapsed time: {elapsed_time:.2f} seconds.'
            login_status = f"未连接到校园网，本次用时 {elapsed_time:.2f} 秒"
            log_manager.log_event('WARNING', log_message)
            show_notification("校园网状态", login_status)
            
    except Exception as e:
        elapsed_time = time.time() - start_time
        log_message = f'Network request failed: {str(e)}. Elapsed time: {elapsed_time:.2f} seconds.'
        log_manager.log_event('ERROR', log_message)
        login_status = f"网络请求失败，本次用时 {elapsed_time:.2f} 秒"
        show_notification("校园网状态", login_status)

except Exception as e:
    log_manager.log_event('ERROR', f'An unexpected error occurred: {str(e)}')