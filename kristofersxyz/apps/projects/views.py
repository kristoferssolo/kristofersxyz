from django.shortcuts import render


def projects(request):
    return render(request, "projects/projects.html", {"title": "Projects"})


def karbs(request):
    return render(request, "projects/karbs/karbs", {"title": "KARBS"})


def instructions(request):
    return render(request, "projects/karbs/instructions.html", {"title": "Karbs Instructions"})
