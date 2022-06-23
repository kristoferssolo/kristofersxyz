from django.shortcuts import render


def index(request):
	return render(request, 'main/index.html', {'title': 'Homepage'})


def lighsaber(request):
	return render(request, 'main/lightsaber.html', {'title': 'Lightsaber'})