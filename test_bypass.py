import re

content = "pub  fn my_func() { if x {} }"
match = re.search(r'pub fn ([a-z0-9_]+)', content)
print("Match 1:", match)

content2 = "pub fn my_func() { if x {} }"
match2 = re.search(r'pub fn ([a-z0-9_]+)', content2)
print("Match 2:", match2)
