from django.shortcuts import redirect, render


def index(request):
    return render(request, "main/index.html", {"title": "Kristofers Solo Webpage"})


def lightsaber(request):
    return render(request, "main/lightsaber.html", {"title": "Lightsaber"})


def projects(request):
    return render(request, "main/projects.html", {"title": "Projects"})


def karbs(request):
    """Karbs install script"""
    return redirect("/projects/karbs/")
