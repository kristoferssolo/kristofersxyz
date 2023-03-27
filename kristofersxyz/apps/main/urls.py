from django.urls import path

from . import views

urlpatterns = [
    path("", views.index, name="home"),
    path("karbs/", views.karbs, name="karbs"),
    path("lightsaber/", views.lightsaber, name="lightsaber"),
]
