from django.shortcuts import render


def index(request):
    """Homepage"""
    return render(request, 'main/index.html', {'title': 'Kristofers Solo Webpage'})
