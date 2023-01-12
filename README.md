# Хз че там с твоими панельками я пока напишу формулы свои сюда

nutrition can be 0-100 (meaning %); <br />
1 nutrition = 8 food * population / 10; <br />
every tick = -10% nutrition + needed; <br />
needed = (100% - nutrition) * 8 food * population / 10; <br />

Pop growth = (nutrition - 50) / 10000 (float) <br />
Every tick = population * pop growth <br />