from django.shortcuts import render


def index(request):
    """Homepage"""
    return render(request, "main/index.html", {"title": "Kristofers Solo Webpage"})


def lightsaber(request):
    """Lightsaber page"""
    return render(request, "main/lightsaber.html", {"title": "Lightsaber"})
