from django.shortcuts import render


def projects(request):
    return render(request, "projects.html", {"title": "Projects"})


def karbs(request):
    return render(request, "karbs/karbs/karbs", {"title": "KARBS"})


def instructions(request):
    return render(request, "karbs/instructions.html", {"title": "Karbs Instructions"})


def traffic_light_detector(request):
    return render(request, "traffic_light_detector/traffic_light_detector.html", {"title": "Traffic Light Detector"})
