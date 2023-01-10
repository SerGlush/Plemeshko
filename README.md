#Хз че там с твоими панельками я пока напишу формулы свои сюда

nutrition can be 0-100 (meaning %);
1 nutrition = 8 food * population / 10;
every tick = -10% nutrition + needed;
needed = (100% - nutrition) * 8 food * population / 10;

Pop growth = (nutrition - 50) / 10000 (float)
Every tick = population * pop growth