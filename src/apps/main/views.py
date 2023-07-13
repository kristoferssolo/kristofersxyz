from django.shortcuts import redirect, render


def index(request):
    return render(request, "index.html", {"title": "Kristofers Solo Webpage"})


def lightsaber(request):
    return render(request, "lightsaber.html", {"title": "Lightsaber"})


def projects(request):
    return render(request, "projects.html", {"title": "Projects"})


def karbs(request):
    return redirect("karbs")
