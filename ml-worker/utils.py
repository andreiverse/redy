from enum import Enum

from bs4 import BeautifulSoup

class Status(Enum):
    INVALID = -1
    SUCCESS = 0
    FAILED = 1
    
def extract_text_from_html_content(html_content: str) -> str:
    return BeautifulSoup(html_content, "html.parser").get_text()