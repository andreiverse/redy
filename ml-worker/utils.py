from enum import Enum

from bs4 import BeautifulSoup

class Status(Enum):
    INVALID = -1
    SUCCESS = 0
    FAILED = 1
    
def extract_text_from_html_content(html_content: str) -> str:
    soup = BeautifulSoup(html_content, "html.parser")
    for script_or_style in soup(["script", "style"]):
        script_or_style.decompose()
    return soup.get_text(separator=" ", strip=True)