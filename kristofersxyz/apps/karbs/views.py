from django.shortcuts import render, redirect


def instructions(request):
    """Karbs Instruction"""
    return render(request, "karbs/instructions.html", {"title": "Karbs Instructions"})


def karbs(request):
    """Karbs install script"""
    return render(request, "karbs/karbs", {"title": "KARBS"})
